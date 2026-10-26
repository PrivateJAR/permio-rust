//! Cursor pagination as a [`Stream`].

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

use futures_core::Stream;
use futures_util::{stream, StreamExt, TryStreamExt};

use crate::error::Error;
use crate::raw::pagination::{PaginationRequest, PaginationResponse};

/// Largest page the API accepts (`PaginationRequest.limit` is 1..=100).
pub const MAX_PAGE_SIZE: u32 = 100;

/// A stream of list items. Every page after the first is fetched lazily as the
/// stream is polled; dropping it stops the listing.
///
/// ```no_run
/// # async fn demo(client: permio::Client) -> Result<(), permio::Error> {
/// use futures_util::TryStreamExt;
/// let roles: Vec<_> = client.list_roles(Default::default()).try_collect().await?;
/// # Ok(()) }
/// ```
pub type ListStream<T> = Pin<Box<dyn Stream<Item = Result<T, Error>> + Send + 'static>>;

/// Options for list calls.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListOptions {
    /// Items requested per page, 1..=100. Default 100.
    pub page_size: Option<u32>,
    /// Filters by name, where the RPC supports it (permissions, roles,
    /// principals, groups). Ignored elsewhere.
    pub search: Option<String>,
}

impl ListOptions {
    /// Sets the page size (clamped to 1..=100).
    #[must_use]
    pub fn page_size(mut self, size: u32) -> Self {
        self.page_size = Some(size);
        self
    }

    /// Filters by name.
    #[must_use]
    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    pub(crate) fn limit(&self) -> i32 {
        let size = self
            .page_size
            .unwrap_or(MAX_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE);
        i32::try_from(size).unwrap_or(100)
    }
}

/// Streams every item of a list RPC that uses `pagination.PaginationRequest`
/// / `PaginationResponse`, following `next_cursor` until the API returns an
/// empty (or repeated) one. Useful with the raw clients:
///
/// ```no_run
/// # async fn demo(client: permio::Client) -> Result<(), permio::Error> {
/// use futures_util::TryStreamExt;
/// use permio::raw::organisation::v1::ListApiKeysRequest;
/// let raw = client.raw_organisations();
/// let keys: Vec<_> = permio::paginate(100, move |pagination| {
///     let mut raw = raw.clone();
///     async move {
///         let res = raw
///             .list_api_keys(ListApiKeysRequest {
///                 project_name: "my-app".into(),
///                 pagination: Some(pagination),
///                 ..Default::default()
///             })
///             .await?
///             .into_inner();
///         Ok((res.api_keys, res.pagination))
///     }
/// })
/// .try_collect()
/// .await?;
/// # Ok(()) }
/// ```
pub fn paginate<T, F, Fut>(page_size: u32, fetch: F) -> ListStream<T>
where
    T: Send + 'static,
    F: FnMut(PaginationRequest) -> Fut + Send + 'static,
    Fut: Future<Output = Result<(Vec<T>, Option<PaginationResponse>), Error>> + Send + 'static,
{
    let limit = ListOptions {
        page_size: Some(page_size),
        search: None,
    }
    .limit();
    struct State<F> {
        fetch: F,
        cursor: Option<String>,
        seen: HashSet<String>,
        done: bool,
    }
    let state = State {
        fetch,
        cursor: None,
        seen: HashSet::new(),
        done: false,
    };
    let pages = stream::try_unfold(state, move |mut state| async move {
        if state.done {
            return Ok::<_, Error>(None);
        }
        let request = PaginationRequest {
            limit: Some(limit),
            cursor: state.cursor.take(),
        };
        let (items, pagination) = (state.fetch)(request).await?;
        let next = pagination.map(|p| p.next_cursor).unwrap_or_default();
        // An empty cursor ends the listing. A repeated one would loop forever,
        // so stop there too.
        if next.is_empty() || !state.seen.insert(next.clone()) {
            state.done = true;
        } else {
            state.cursor = Some(next);
        }
        Ok(Some((items, state)))
    });
    pages
        .map_ok(|items| stream::iter(items.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
}

/// A stream that yields `err` once.
pub(crate) fn failed<T: Send + 'static>(err: Error) -> ListStream<T> {
    stream::once(async move { Err(err) }).boxed()
}
