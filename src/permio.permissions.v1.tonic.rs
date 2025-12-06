// @generated
/// Generated client implementations.
pub mod permissions_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct PermissionsServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl PermissionsServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> PermissionsServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> PermissionsServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            PermissionsServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn create_permission(
            &mut self,
            request: impl tonic::IntoRequest<super::CreatePermissionRequest>,
        ) -> std::result::Result<tonic::Response<super::Permission>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/CreatePermission",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "CreatePermission",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_permission(
            &mut self,
            request: impl tonic::IntoRequest<super::GetPermissionRequest>,
        ) -> std::result::Result<tonic::Response<super::Permission>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/GetPermission",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "GetPermission",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_permission(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdatePermissionRequest>,
        ) -> std::result::Result<tonic::Response<super::Permission>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/UpdatePermission",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "UpdatePermission",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_permission(
            &mut self,
            request: impl tonic::IntoRequest<super::DeletePermissionRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/DeletePermission",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "DeletePermission",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_permissions(
            &mut self,
            request: impl tonic::IntoRequest<super::ListPermissionsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListPermissionsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/ListPermissions",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "ListPermissions",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_role(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateRoleRequest>,
        ) -> std::result::Result<tonic::Response<super::Role>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/CreateRole",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "CreateRole",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_role(
            &mut self,
            request: impl tonic::IntoRequest<super::GetRoleRequest>,
        ) -> std::result::Result<tonic::Response<super::Role>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/GetRole",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "GetRole",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_role(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateRoleRequest>,
        ) -> std::result::Result<tonic::Response<super::Role>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/UpdateRole",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "UpdateRole",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_role(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteRoleRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/DeleteRole",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "DeleteRole",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_roles(
            &mut self,
            request: impl tonic::IntoRequest<super::ListRolesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListRolesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/ListRoles",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "ListRoles",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn grant_user_permissions_and_roles_on_resource(
            &mut self,
            request: impl tonic::IntoRequest<
                super::GrantUserPermissionsAndRolesOnResourceRequest,
            >,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/GrantUserPermissionsAndRolesOnResource",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "GrantUserPermissionsAndRolesOnResource",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn revoke_user_permissions_and_roles_on_resource(
            &mut self,
            request: impl tonic::IntoRequest<
                super::RevokeUserPermissionsAndRolesOnResourceRequest,
            >,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/RevokeUserPermissionsAndRolesOnResource",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "RevokeUserPermissionsAndRolesOnResource",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_user_permissions_and_roles_on_resource(
            &mut self,
            request: impl tonic::IntoRequest<
                super::GetUserPermissionsAndRolesOnResourceRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::GetUserPermissionsAndRolesOnResourceResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/GetUserPermissionsAndRolesOnResource",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "GetUserPermissionsAndRolesOnResource",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_principals(
            &mut self,
            request: impl tonic::IntoRequest<super::ListPrincipalsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListPrincipalsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/ListPrincipals",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "ListPrincipals",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_all_assignments_for_principal(
            &mut self,
            request: impl tonic::IntoRequest<super::GetAllAssignmentsForPrincipalRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetAllAssignmentsForPrincipalResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/GetAllAssignmentsForPrincipal",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "GetAllAssignmentsForPrincipal",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn check(
            &mut self,
            request: impl tonic::IntoRequest<super::CheckPermissionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CheckPermissionResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/Check",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("permio.permissions.v1.PermissionsService", "Check"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_total_roles_and_permissions_for_organisation(
            &mut self,
            request: impl tonic::IntoRequest<
                super::GetTotalRolesAndPermissionsForOrganisationRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::GetTotalRolesAndPermissionsForOrganisationResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/permio.permissions.v1.PermissionsService/GetTotalRolesAndPermissionsForOrganisation",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.permissions.v1.PermissionsService",
                        "GetTotalRolesAndPermissionsForOrganisation",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod permissions_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with PermissionsServiceServer.
    #[async_trait]
    pub trait PermissionsService: Send + Sync + 'static {
        async fn create_permission(
            &self,
            request: tonic::Request<super::CreatePermissionRequest>,
        ) -> std::result::Result<tonic::Response<super::Permission>, tonic::Status>;
        async fn get_permission(
            &self,
            request: tonic::Request<super::GetPermissionRequest>,
        ) -> std::result::Result<tonic::Response<super::Permission>, tonic::Status>;
        async fn update_permission(
            &self,
            request: tonic::Request<super::UpdatePermissionRequest>,
        ) -> std::result::Result<tonic::Response<super::Permission>, tonic::Status>;
        async fn delete_permission(
            &self,
            request: tonic::Request<super::DeletePermissionRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        async fn list_permissions(
            &self,
            request: tonic::Request<super::ListPermissionsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListPermissionsResponse>,
            tonic::Status,
        >;
        async fn create_role(
            &self,
            request: tonic::Request<super::CreateRoleRequest>,
        ) -> std::result::Result<tonic::Response<super::Role>, tonic::Status>;
        async fn get_role(
            &self,
            request: tonic::Request<super::GetRoleRequest>,
        ) -> std::result::Result<tonic::Response<super::Role>, tonic::Status>;
        async fn update_role(
            &self,
            request: tonic::Request<super::UpdateRoleRequest>,
        ) -> std::result::Result<tonic::Response<super::Role>, tonic::Status>;
        async fn delete_role(
            &self,
            request: tonic::Request<super::DeleteRoleRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        async fn list_roles(
            &self,
            request: tonic::Request<super::ListRolesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListRolesResponse>,
            tonic::Status,
        >;
        async fn grant_user_permissions_and_roles_on_resource(
            &self,
            request: tonic::Request<super::GrantUserPermissionsAndRolesOnResourceRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        async fn revoke_user_permissions_and_roles_on_resource(
            &self,
            request: tonic::Request<
                super::RevokeUserPermissionsAndRolesOnResourceRequest,
            >,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        async fn get_user_permissions_and_roles_on_resource(
            &self,
            request: tonic::Request<super::GetUserPermissionsAndRolesOnResourceRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetUserPermissionsAndRolesOnResourceResponse>,
            tonic::Status,
        >;
        async fn list_principals(
            &self,
            request: tonic::Request<super::ListPrincipalsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListPrincipalsResponse>,
            tonic::Status,
        >;
        async fn get_all_assignments_for_principal(
            &self,
            request: tonic::Request<super::GetAllAssignmentsForPrincipalRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetAllAssignmentsForPrincipalResponse>,
            tonic::Status,
        >;
        async fn check(
            &self,
            request: tonic::Request<super::CheckPermissionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CheckPermissionResponse>,
            tonic::Status,
        >;
        async fn get_total_roles_and_permissions_for_organisation(
            &self,
            request: tonic::Request<
                super::GetTotalRolesAndPermissionsForOrganisationRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::GetTotalRolesAndPermissionsForOrganisationResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct PermissionsServiceServer<T: PermissionsService> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T: PermissionsService> PermissionsServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for PermissionsServiceServer<T>
    where
        T: PermissionsService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/permio.permissions.v1.PermissionsService/CreatePermission" => {
                    #[allow(non_camel_case_types)]
                    struct CreatePermissionSvc<T: PermissionsService>(pub Arc<T>);
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<super::CreatePermissionRequest>
                    for CreatePermissionSvc<T> {
                        type Response = super::Permission;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreatePermissionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::create_permission(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreatePermissionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/GetPermission" => {
                    #[allow(non_camel_case_types)]
                    struct GetPermissionSvc<T: PermissionsService>(pub Arc<T>);
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<super::GetPermissionRequest>
                    for GetPermissionSvc<T> {
                        type Response = super::Permission;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetPermissionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::get_permission(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetPermissionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/UpdatePermission" => {
                    #[allow(non_camel_case_types)]
                    struct UpdatePermissionSvc<T: PermissionsService>(pub Arc<T>);
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<super::UpdatePermissionRequest>
                    for UpdatePermissionSvc<T> {
                        type Response = super::Permission;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdatePermissionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::update_permission(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = UpdatePermissionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/DeletePermission" => {
                    #[allow(non_camel_case_types)]
                    struct DeletePermissionSvc<T: PermissionsService>(pub Arc<T>);
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<super::DeletePermissionRequest>
                    for DeletePermissionSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeletePermissionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::delete_permission(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = DeletePermissionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/ListPermissions" => {
                    #[allow(non_camel_case_types)]
                    struct ListPermissionsSvc<T: PermissionsService>(pub Arc<T>);
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<super::ListPermissionsRequest>
                    for ListPermissionsSvc<T> {
                        type Response = super::ListPermissionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListPermissionsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::list_permissions(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListPermissionsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/CreateRole" => {
                    #[allow(non_camel_case_types)]
                    struct CreateRoleSvc<T: PermissionsService>(pub Arc<T>);
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<super::CreateRoleRequest>
                    for CreateRoleSvc<T> {
                        type Response = super::Role;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateRoleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::create_role(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreateRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/GetRole" => {
                    #[allow(non_camel_case_types)]
                    struct GetRoleSvc<T: PermissionsService>(pub Arc<T>);
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<super::GetRoleRequest>
                    for GetRoleSvc<T> {
                        type Response = super::Role;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetRoleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::get_role(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/UpdateRole" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateRoleSvc<T: PermissionsService>(pub Arc<T>);
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<super::UpdateRoleRequest>
                    for UpdateRoleSvc<T> {
                        type Response = super::Role;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateRoleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::update_role(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = UpdateRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/DeleteRole" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteRoleSvc<T: PermissionsService>(pub Arc<T>);
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<super::DeleteRoleRequest>
                    for DeleteRoleSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteRoleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::delete_role(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = DeleteRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/ListRoles" => {
                    #[allow(non_camel_case_types)]
                    struct ListRolesSvc<T: PermissionsService>(pub Arc<T>);
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<super::ListRolesRequest>
                    for ListRolesSvc<T> {
                        type Response = super::ListRolesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListRolesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::list_roles(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListRolesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/GrantUserPermissionsAndRolesOnResource" => {
                    #[allow(non_camel_case_types)]
                    struct GrantUserPermissionsAndRolesOnResourceSvc<
                        T: PermissionsService,
                    >(
                        pub Arc<T>,
                    );
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<
                        super::GrantUserPermissionsAndRolesOnResourceRequest,
                    > for GrantUserPermissionsAndRolesOnResourceSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GrantUserPermissionsAndRolesOnResourceRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::grant_user_permissions_and_roles_on_resource(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GrantUserPermissionsAndRolesOnResourceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/RevokeUserPermissionsAndRolesOnResource" => {
                    #[allow(non_camel_case_types)]
                    struct RevokeUserPermissionsAndRolesOnResourceSvc<
                        T: PermissionsService,
                    >(
                        pub Arc<T>,
                    );
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<
                        super::RevokeUserPermissionsAndRolesOnResourceRequest,
                    > for RevokeUserPermissionsAndRolesOnResourceSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::RevokeUserPermissionsAndRolesOnResourceRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::revoke_user_permissions_and_roles_on_resource(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = RevokeUserPermissionsAndRolesOnResourceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/GetUserPermissionsAndRolesOnResource" => {
                    #[allow(non_camel_case_types)]
                    struct GetUserPermissionsAndRolesOnResourceSvc<
                        T: PermissionsService,
                    >(
                        pub Arc<T>,
                    );
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<
                        super::GetUserPermissionsAndRolesOnResourceRequest,
                    > for GetUserPermissionsAndRolesOnResourceSvc<T> {
                        type Response = super::GetUserPermissionsAndRolesOnResourceResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GetUserPermissionsAndRolesOnResourceRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::get_user_permissions_and_roles_on_resource(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetUserPermissionsAndRolesOnResourceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/ListPrincipals" => {
                    #[allow(non_camel_case_types)]
                    struct ListPrincipalsSvc<T: PermissionsService>(pub Arc<T>);
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<super::ListPrincipalsRequest>
                    for ListPrincipalsSvc<T> {
                        type Response = super::ListPrincipalsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListPrincipalsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::list_principals(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListPrincipalsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/GetAllAssignmentsForPrincipal" => {
                    #[allow(non_camel_case_types)]
                    struct GetAllAssignmentsForPrincipalSvc<T: PermissionsService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<
                        super::GetAllAssignmentsForPrincipalRequest,
                    > for GetAllAssignmentsForPrincipalSvc<T> {
                        type Response = super::GetAllAssignmentsForPrincipalResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GetAllAssignmentsForPrincipalRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::get_all_assignments_for_principal(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetAllAssignmentsForPrincipalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/Check" => {
                    #[allow(non_camel_case_types)]
                    struct CheckSvc<T: PermissionsService>(pub Arc<T>);
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<super::CheckPermissionRequest>
                    for CheckSvc<T> {
                        type Response = super::CheckPermissionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CheckPermissionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::check(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CheckSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/permio.permissions.v1.PermissionsService/GetTotalRolesAndPermissionsForOrganisation" => {
                    #[allow(non_camel_case_types)]
                    struct GetTotalRolesAndPermissionsForOrganisationSvc<
                        T: PermissionsService,
                    >(
                        pub Arc<T>,
                    );
                    impl<
                        T: PermissionsService,
                    > tonic::server::UnaryService<
                        super::GetTotalRolesAndPermissionsForOrganisationRequest,
                    > for GetTotalRolesAndPermissionsForOrganisationSvc<T> {
                        type Response = super::GetTotalRolesAndPermissionsForOrganisationResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GetTotalRolesAndPermissionsForOrganisationRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PermissionsService>::get_total_roles_and_permissions_for_organisation(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetTotalRolesAndPermissionsForOrganisationSvc(
                            inner,
                        );
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", tonic::Code::Unimplemented as i32)
                                .header(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                )
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: PermissionsService> Clone for PermissionsServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: PermissionsService> tonic::server::NamedService
    for PermissionsServiceServer<T> {
        const NAME: &'static str = "permio.permissions.v1.PermissionsService";
    }
}
