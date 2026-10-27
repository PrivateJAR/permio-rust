//! Cursor pagination as a [`Stream`].

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

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
///
/// ```
/// use std::time::Duration;
/// let options = permio::ListOptions::new()
///     .page_size(50)
///     .project("other-app")
///     .timeout(Duration::from_secs(5));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListOptions {
    /// Items requested per page, 1..=100. Default 100.
    pub page_size: Option<u32>,
    /// Filters by name, where the RPC supports it (permissions, roles,
    /// principals, groups). Ignored elsewhere.
    pub search: Option<String>,
    /// Lists in this project instead of the client's default one.
    pub project: Option<String>,
    /// Overall deadline for each page request, across its retries. Defaults
    /// to the client's [`call_timeout`](crate::ClientBuilder::call_timeout).
    pub timeout: Option<Duration>,
}

impl ListOptions {
    /// Default options: pages of 100, no search, the client's project.
    #[must_use]
    pub fn new() -> Self {
        ListOptions::default()
    }

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

    /// Lists in `project` instead of the client's default project.
    #[must_use]
    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Overall deadline for each page request, across its retries.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
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
/// empty one. A cursor the API already returned would loop forever, so the
/// stream then ends with [`Error::Other`]. Useful with the raw clients:
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
    paginate_capped(page_size, None, |items: &Vec<T>| items.len(), fetch)
        .map_ok(|items| stream::iter(items.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
}

/// Streams the pages of a list RPC. With `budget`, each page request asks for
/// at most the items still in the budget (as counted by `count`), and the
/// stream ends once the budget is used up, so at most `budget` items are ever
/// requested.
pub(crate) fn paginate_capped<P, F, Fut, C>(
    page_size: u32,
    budget: Option<usize>,
    count: C,
    fetch: F,
) -> ListStream<P>
where
    P: Send + 'static,
    F: FnMut(PaginationRequest) -> Fut + Send + 'static,
    Fut: Future<Output = Result<(P, Option<PaginationResponse>), Error>> + Send + 'static,
    C: Fn(&P) -> usize + Send + 'static,
{
    let limit = ListOptions::new().page_size(page_size).limit();
    struct State<F, C> {
        fetch: F,
        count: C,
        cursor: Option<String>,
        seen: HashSet<String>,
        remaining: Option<usize>,
        done: bool,
    }
    let state = State {
        fetch,
        count,
        cursor: None,
        seen: HashSet::new(),
        remaining: budget,
        done: false,
    };
    stream::try_unfold(state, move |mut state| async move {
        if state.done || state.remaining == Some(0) {
            return Ok::<_, Error>(None);
        }
        let limit = match state.remaining {
            Some(r) => limit.min(i32::try_from(r).unwrap_or(i32::MAX)),
            None => limit,
        };
        let request = PaginationRequest {
            limit: Some(limit),
            cursor: state.cursor.take(),
        };
        let (page, pagination) = (state.fetch)(request).await?;
        if let Some(r) = state.remaining.as_mut() {
            *r = r.saturating_sub((state.count)(&page));
        }
        let next = pagination.map(|p| p.next_cursor).unwrap_or_default();
        // An empty cursor ends the listing. A repeated one would loop forever;
        // ending quietly would return a truncated listing as if complete.
        if next.is_empty() {
            state.done = true;
        } else if !state.seen.insert(next.clone()) {
            return Err(Error::inconsistent(format!(
                "pagination cursor {next:?} was returned twice"
            )));
        } else {
            state.cursor = Some(next);
        }
        Ok(Some((page, state)))
    })
    .boxed()
}

/// Collects `stream` into a Vec, failing with [`Error::TooManyResults`] as
/// soon as it yields more than `max_results` items.
pub(crate) async fn collect_capped<T>(
    mut stream: ListStream<T>,
    max_results: usize,
    call: &str,
    streaming: &str,
) -> Result<Vec<T>, Error> {
    let mut out = Vec::new();
    while let Some(item) = stream.try_next().await? {
        if out.len() >= max_results {
            return Err(Error::too_many_results(call, streaming, max_results));
        }
        out.push(item);
    }
    Ok(out)
}

/// A stream that yields `err` once.
pub(crate) fn failed<T: Send + 'static>(err: Error) -> ListStream<T> {
    stream::once(async move { Err(err) }).boxed()
}
