//! An in-process PermissionsService for unit tests. Every RPC is recorded and
//! answered by a handler closure; unhandled RPCs return the default response.
#![allow(dead_code)]

use std::any::Any;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use permio::raw::permissions::v1::permissions_service_server::{
    PermissionsService, PermissionsServiceServer,
};
use permio::raw::permissions::v1::*;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Endpoint, Server};
use tonic::{Request, Response, Status};

/// A valid opaque test key (same fixture as permio-js).
pub const KEY: &str = "perms_0123456789ABCDEFGHIJKLMNOPQRSTUV1ggZdL";

/// How the fake answers one call.
pub enum Reply {
    /// The response type's default value.
    Default,
    /// This response (must be the RPC's response type).
    Ok(Box<dyn Any + Send>),
    /// This error.
    Err(Status),
    /// Waits, then answers.
    Sleep(Duration, Box<Reply>),
}

impl Reply {
    pub fn ok<T: Any + Send>(value: T) -> Reply {
        Reply::Ok(Box::new(value))
    }
}

/// One recorded call.
#[derive(Clone)]
pub struct Call {
    pub method: &'static str,
    pub authorization: Option<String>,
    pub request: Arc<dyn Any + Send + Sync>,
}

impl Call {
    pub fn request<T: 'static>(&self) -> &T {
        self.request.downcast_ref::<T>().expect("request type")
    }
}

/// `(method, request, attempt index for that method) -> Reply`.
pub type Handler = Arc<dyn Fn(&str, &dyn Any, usize) -> Reply + Send + Sync>;

#[derive(Clone)]
pub struct Fake {
    pub calls: Arc<Mutex<Vec<Call>>>,
    handler: Handler,
}

impl Fake {
    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().unwrap().clone()
    }

    pub fn methods(&self) -> Vec<&'static str> {
        self.calls().iter().map(|c| c.method).collect()
    }

    async fn answer<Req, Res>(
        &self,
        method: &'static str,
        request: Request<Req>,
    ) -> Result<Response<Res>, Status>
    where
        Req: Any + Send + Sync,
        Res: Any + Default,
    {
        let authorization = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let req = Arc::new(request.into_inner());
        let attempt = {
            let mut calls = self.calls.lock().unwrap();
            let n = calls.iter().filter(|c| c.method == method).count();
            calls.push(Call {
                method,
                authorization,
                request: req.clone(),
            });
            n
        };
        let mut reply = (self.handler)(method, &*req as &dyn Any, attempt);
        loop {
            match reply {
                Reply::Default => return Ok(Response::new(Res::default())),
                Reply::Ok(v) => {
                    let v = v
                        .downcast::<Res>()
                        .unwrap_or_else(|_| panic!("handler returned the wrong type for {method}"));
                    return Ok(Response::new(*v));
                }
                Reply::Err(s) => return Err(s),
                Reply::Sleep(d, next) => {
                    tokio::time::sleep(d).await;
                    reply = *next;
                }
            }
        }
    }
}

macro_rules! fake_methods {
    ($($name:ident: $req:ty => $res:ty, $grpc:literal;)*) => {
        #[tonic::async_trait]
        impl PermissionsService for Fake {
            $(
                async fn $name(&self, request: Request<$req>) -> Result<Response<$res>, Status> {
                    self.answer($grpc, request).await
                }
            )*
        }
    };
}

fake_methods! {
    create_permission: CreatePermissionRequest => Permission, "CreatePermission";
    get_permission: GetPermissionRequest => Permission, "GetPermission";
    update_permission: UpdatePermissionRequest => Permission, "UpdatePermission";
    delete_permission: DeletePermissionRequest => (), "DeletePermission";
    list_permissions: ListPermissionsRequest => ListPermissionsResponse, "ListPermissions";
    create_role: CreateRoleRequest => Role, "CreateRole";
    get_role: GetRoleRequest => Role, "GetRole";
    update_role: UpdateRoleRequest => Role, "UpdateRole";
    delete_role: DeleteRoleRequest => (), "DeleteRole";
    list_roles: ListRolesRequest => ListRolesResponse, "ListRoles";
    grant_user_permissions_and_roles_on_resource: GrantUserPermissionsAndRolesOnResourceRequest => (), "Grant";
    revoke_user_permissions_and_roles_on_resource: RevokeUserPermissionsAndRolesOnResourceRequest => (), "Revoke";
    get_user_permissions_and_roles_on_resource: GetUserPermissionsAndRolesOnResourceRequest => GetUserPermissionsAndRolesOnResourceResponse, "GetUserPermissionsAndRolesOnResource";
    list_principals: ListPrincipalsRequest => ListPrincipalsResponse, "ListPrincipals";
    get_all_assignments_for_principal: GetAllAssignmentsForPrincipalRequest => GetAllAssignmentsForPrincipalResponse, "GetAllAssignmentsForPrincipal";
    check: CheckPermissionRequest => CheckPermissionResponse, "Check";
    get_total_roles_and_permissions_for_organisation: GetTotalRolesAndPermissionsForOrganisationRequest => GetTotalRolesAndPermissionsForOrganisationResponse, "GetTotalRolesAndPermissionsForOrganisation";
    batch_check: BatchCheckRequest => BatchCheckResponse, "BatchCheck";
    list_access: ListAccessRequest => ListAccessResponse, "ListAccess";
    list_principals_with_access: ListPrincipalsWithAccessRequest => ListPrincipalsWithAccessResponse, "ListPrincipalsWithAccess";
    create_group: CreateGroupRequest => Group, "CreateGroup";
    get_group: GetGroupRequest => Group, "GetGroup";
    update_group: UpdateGroupRequest => Group, "UpdateGroup";
    delete_group: DeleteGroupRequest => (), "DeleteGroup";
    list_groups: ListGroupsRequest => ListGroupsResponse, "ListGroups";
    add_group_members: AddGroupMembersRequest => AddGroupMembersResponse, "AddGroupMembers";
    remove_group_members: RemoveGroupMembersRequest => RemoveGroupMembersResponse, "RemoveGroupMembers";
    list_group_members: ListGroupMembersRequest => ListGroupMembersResponse, "ListGroupMembers";
    list_groups_for_principal: ListGroupsForPrincipalRequest => ListGroupsForPrincipalResponse, "ListGroupsForPrincipal";
    who_am_i: WhoAmIRequest => WhoAmIResponse, "WhoAmI";
}

/// Starts the fake on a local port and returns it with a builder whose
/// channel points at it (API key and project "proj" set, fast retries).
pub async fn start<F>(handler: F) -> (Fake, permio::ClientBuilder)
where
    F: Fn(&str, &dyn Any, usize) -> Reply + Send + Sync + 'static,
{
    let (fake, channel) = serve(handler).await;
    let builder = permio::Client::builder()
        .api_key(KEY)
        .project("proj")
        .channel(channel)
        .retry_config(permio::RetryConfig {
            max_retries: 2,
            base_delay: Duration::from_millis(5),
            max_delay: Duration::from_millis(200),
        });
    (fake, builder)
}

/// Starts the fake on a local port and returns a channel connected to it.
pub async fn serve<F>(handler: F) -> (Fake, Channel)
where
    F: Fn(&str, &dyn Any, usize) -> Reply + Send + Sync + 'static,
{
    let fake = Fake {
        calls: Arc::new(Mutex::new(Vec::new())),
        handler: Arc::new(handler),
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let service = PermissionsServiceServer::new(fake.clone());
    tokio::spawn(async move {
        Server::builder()
            .add_service(service)
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    });
    let channel = Endpoint::from_shared(format!("http://{addr}"))
        .unwrap()
        .connect_lazy();
    (fake, channel)
}

/// A client with the default (always default-response) handler.
pub async fn default_client() -> (Fake, permio::Client) {
    let (fake, builder) = start(|_, _, _| Reply::Default).await;
    (fake, builder.build().unwrap())
}
