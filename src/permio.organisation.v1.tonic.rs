// @generated
/// Generated client implementations.
pub mod organisation_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct OrganisationServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl OrganisationServiceClient<tonic::transport::Channel> {
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
    impl<T> OrganisationServiceClient<T>
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
        ) -> OrganisationServiceClient<InterceptedService<T, F>>
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
            OrganisationServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn create_organisation(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateOrganisationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateOrganisationResponse>,
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
                "/permio.organisation.v1.OrganisationService/CreateOrganisation",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "CreateOrganisation",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_organisation(
            &mut self,
            request: impl tonic::IntoRequest<super::GetOrganisationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetOrganisationResponse>,
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
                "/permio.organisation.v1.OrganisationService/GetOrganisation",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "GetOrganisation",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_organisation(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateOrganisationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateOrganisationResponse>,
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
                "/permio.organisation.v1.OrganisationService/UpdateOrganisation",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "UpdateOrganisation",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn set_users_organisation(
            &mut self,
            request: impl tonic::IntoRequest<super::SetUsersOrganisationRequest>,
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
                "/permio.organisation.v1.OrganisationService/SetUsersOrganisation",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "SetUsersOrganisation",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn invite_user_to_organisation(
            &mut self,
            request: impl tonic::IntoRequest<super::InviteUserToOrganisationRequest>,
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
                "/permio.organisation.v1.OrganisationService/InviteUserToOrganisation",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "InviteUserToOrganisation",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_users_organisation(
            &mut self,
            request: impl tonic::IntoRequest<super::GetUsersOrganisationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetOrganisationResponse>,
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
                "/permio.organisation.v1.OrganisationService/GetUsersOrganisation",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "GetUsersOrganisation",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_api_key(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateApiKeyRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateApiKeyResponse>,
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
                "/permio.organisation.v1.OrganisationService/CreateAPIKey",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "CreateAPIKey",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_api_keys(
            &mut self,
            request: impl tonic::IntoRequest<super::ListApiKeysRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListApiKeysResponse>,
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
                "/permio.organisation.v1.OrganisationService/ListAPIKeys",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "ListAPIKeys",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn invalidate_api_key(
            &mut self,
            request: impl tonic::IntoRequest<super::InvalidateApiKeyRequest>,
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
                "/permio.organisation.v1.OrganisationService/InvalidateAPIKey",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "InvalidateAPIKey",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_project(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateProjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateProjectResponse>,
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
                "/permio.organisation.v1.OrganisationService/CreateProject",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "CreateProject",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_project(
            &mut self,
            request: impl tonic::IntoRequest<super::GetProjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetProjectResponse>,
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
                "/permio.organisation.v1.OrganisationService/GetProject",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "GetProject",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_projects_for_organisation(
            &mut self,
            request: impl tonic::IntoRequest<super::GetProjectsForOrganisationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetProjectsForOrganisationResponse>,
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
                "/permio.organisation.v1.OrganisationService/GetProjectsForOrganisation",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "GetProjectsForOrganisation",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_project(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteProjectRequest>,
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
                "/permio.organisation.v1.OrganisationService/DeleteProject",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "DeleteProject",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_principal_usage_stats(
            &mut self,
            request: impl tonic::IntoRequest<super::GetPrincipalUsageStatsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetPrincipalUsageStatsResponse>,
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
                "/permio.organisation.v1.OrganisationService/GetPrincipalUsageStats",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "GetPrincipalUsageStats",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_usage_history(
            &mut self,
            request: impl tonic::IntoRequest<super::GetUsageHistoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetUsageHistoryResponse>,
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
                "/permio.organisation.v1.OrganisationService/GetUsageHistory",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "GetUsageHistory",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_organisation_dashboard(
            &mut self,
            request: impl tonic::IntoRequest<super::GetOrganisationDashboardRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetOrganisationDashboardResponse>,
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
                "/permio.organisation.v1.OrganisationService/GetOrganisationDashboard",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "GetOrganisationDashboard",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn remove_user_from_organisation(
            &mut self,
            request: impl tonic::IntoRequest<super::RemoveUserFromOrganisationRequest>,
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
                "/permio.organisation.v1.OrganisationService/RemoveUserFromOrganisation",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "permio.organisation.v1.OrganisationService",
                        "RemoveUserFromOrganisation",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod organisation_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with OrganisationServiceServer.
    #[async_trait]
    pub trait OrganisationService: Send + Sync + 'static {
        async fn create_organisation(
            &self,
            request: tonic::Request<super::CreateOrganisationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateOrganisationResponse>,
            tonic::Status,
        >;
        async fn get_organisation(
            &self,
            request: tonic::Request<super::GetOrganisationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetOrganisationResponse>,
            tonic::Status,
        >;
        async fn update_organisation(
            &self,
            request: tonic::Request<super::UpdateOrganisationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateOrganisationResponse>,
            tonic::Status,
        >;
        async fn set_users_organisation(
            &self,
            request: tonic::Request<super::SetUsersOrganisationRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        async fn invite_user_to_organisation(
            &self,
            request: tonic::Request<super::InviteUserToOrganisationRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        async fn get_users_organisation(
            &self,
            request: tonic::Request<super::GetUsersOrganisationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetOrganisationResponse>,
            tonic::Status,
        >;
        async fn create_api_key(
            &self,
            request: tonic::Request<super::CreateApiKeyRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateApiKeyResponse>,
            tonic::Status,
        >;
        async fn list_api_keys(
            &self,
            request: tonic::Request<super::ListApiKeysRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListApiKeysResponse>,
            tonic::Status,
        >;
        async fn invalidate_api_key(
            &self,
            request: tonic::Request<super::InvalidateApiKeyRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        async fn create_project(
            &self,
            request: tonic::Request<super::CreateProjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateProjectResponse>,
            tonic::Status,
        >;
        async fn get_project(
            &self,
            request: tonic::Request<super::GetProjectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetProjectResponse>,
            tonic::Status,
        >;
        async fn get_projects_for_organisation(
            &self,
            request: tonic::Request<super::GetProjectsForOrganisationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetProjectsForOrganisationResponse>,
            tonic::Status,
        >;
        async fn delete_project(
            &self,
            request: tonic::Request<super::DeleteProjectRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
        async fn get_principal_usage_stats(
            &self,
            request: tonic::Request<super::GetPrincipalUsageStatsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetPrincipalUsageStatsResponse>,
            tonic::Status,
        >;
        async fn get_usage_history(
            &self,
            request: tonic::Request<super::GetUsageHistoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetUsageHistoryResponse>,
            tonic::Status,
        >;
        async fn get_organisation_dashboard(
            &self,
            request: tonic::Request<super::GetOrganisationDashboardRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetOrganisationDashboardResponse>,
            tonic::Status,
        >;
        async fn remove_user_from_organisation(
            &self,
            request: tonic::Request<super::RemoveUserFromOrganisationRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct OrganisationServiceServer<T: OrganisationService> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T: OrganisationService> OrganisationServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for OrganisationServiceServer<T>
    where
        T: OrganisationService,
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
                "/permio.organisation.v1.OrganisationService/CreateOrganisation" => {
                    #[allow(non_camel_case_types)]
                    struct CreateOrganisationSvc<T: OrganisationService>(pub Arc<T>);
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::CreateOrganisationRequest>
                    for CreateOrganisationSvc<T> {
                        type Response = super::CreateOrganisationResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateOrganisationRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::create_organisation(
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
                        let method = CreateOrganisationSvc(inner);
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
                "/permio.organisation.v1.OrganisationService/GetOrganisation" => {
                    #[allow(non_camel_case_types)]
                    struct GetOrganisationSvc<T: OrganisationService>(pub Arc<T>);
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::GetOrganisationRequest>
                    for GetOrganisationSvc<T> {
                        type Response = super::GetOrganisationResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetOrganisationRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::get_organisation(
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
                        let method = GetOrganisationSvc(inner);
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
                "/permio.organisation.v1.OrganisationService/UpdateOrganisation" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateOrganisationSvc<T: OrganisationService>(pub Arc<T>);
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::UpdateOrganisationRequest>
                    for UpdateOrganisationSvc<T> {
                        type Response = super::UpdateOrganisationResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateOrganisationRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::update_organisation(
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
                        let method = UpdateOrganisationSvc(inner);
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
                "/permio.organisation.v1.OrganisationService/SetUsersOrganisation" => {
                    #[allow(non_camel_case_types)]
                    struct SetUsersOrganisationSvc<T: OrganisationService>(pub Arc<T>);
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::SetUsersOrganisationRequest>
                    for SetUsersOrganisationSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetUsersOrganisationRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::set_users_organisation(
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
                        let method = SetUsersOrganisationSvc(inner);
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
                "/permio.organisation.v1.OrganisationService/InviteUserToOrganisation" => {
                    #[allow(non_camel_case_types)]
                    struct InviteUserToOrganisationSvc<T: OrganisationService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::InviteUserToOrganisationRequest>
                    for InviteUserToOrganisationSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::InviteUserToOrganisationRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::invite_user_to_organisation(
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
                        let method = InviteUserToOrganisationSvc(inner);
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
                "/permio.organisation.v1.OrganisationService/GetUsersOrganisation" => {
                    #[allow(non_camel_case_types)]
                    struct GetUsersOrganisationSvc<T: OrganisationService>(pub Arc<T>);
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::GetUsersOrganisationRequest>
                    for GetUsersOrganisationSvc<T> {
                        type Response = super::GetOrganisationResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetUsersOrganisationRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::get_users_organisation(
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
                        let method = GetUsersOrganisationSvc(inner);
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
                "/permio.organisation.v1.OrganisationService/CreateAPIKey" => {
                    #[allow(non_camel_case_types)]
                    struct CreateAPIKeySvc<T: OrganisationService>(pub Arc<T>);
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::CreateApiKeyRequest>
                    for CreateAPIKeySvc<T> {
                        type Response = super::CreateApiKeyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateApiKeyRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::create_api_key(&inner, request)
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
                        let method = CreateAPIKeySvc(inner);
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
                "/permio.organisation.v1.OrganisationService/ListAPIKeys" => {
                    #[allow(non_camel_case_types)]
                    struct ListAPIKeysSvc<T: OrganisationService>(pub Arc<T>);
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::ListApiKeysRequest>
                    for ListAPIKeysSvc<T> {
                        type Response = super::ListApiKeysResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListApiKeysRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::list_api_keys(&inner, request)
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
                        let method = ListAPIKeysSvc(inner);
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
                "/permio.organisation.v1.OrganisationService/InvalidateAPIKey" => {
                    #[allow(non_camel_case_types)]
                    struct InvalidateAPIKeySvc<T: OrganisationService>(pub Arc<T>);
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::InvalidateApiKeyRequest>
                    for InvalidateAPIKeySvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::InvalidateApiKeyRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::invalidate_api_key(
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
                        let method = InvalidateAPIKeySvc(inner);
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
                "/permio.organisation.v1.OrganisationService/CreateProject" => {
                    #[allow(non_camel_case_types)]
                    struct CreateProjectSvc<T: OrganisationService>(pub Arc<T>);
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::CreateProjectRequest>
                    for CreateProjectSvc<T> {
                        type Response = super::CreateProjectResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateProjectRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::create_project(&inner, request)
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
                        let method = CreateProjectSvc(inner);
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
                "/permio.organisation.v1.OrganisationService/GetProject" => {
                    #[allow(non_camel_case_types)]
                    struct GetProjectSvc<T: OrganisationService>(pub Arc<T>);
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::GetProjectRequest>
                    for GetProjectSvc<T> {
                        type Response = super::GetProjectResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetProjectRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::get_project(&inner, request)
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
                        let method = GetProjectSvc(inner);
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
                "/permio.organisation.v1.OrganisationService/GetProjectsForOrganisation" => {
                    #[allow(non_camel_case_types)]
                    struct GetProjectsForOrganisationSvc<T: OrganisationService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<
                        super::GetProjectsForOrganisationRequest,
                    > for GetProjectsForOrganisationSvc<T> {
                        type Response = super::GetProjectsForOrganisationResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GetProjectsForOrganisationRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::get_projects_for_organisation(
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
                        let method = GetProjectsForOrganisationSvc(inner);
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
                "/permio.organisation.v1.OrganisationService/DeleteProject" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteProjectSvc<T: OrganisationService>(pub Arc<T>);
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::DeleteProjectRequest>
                    for DeleteProjectSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteProjectRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::delete_project(&inner, request)
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
                        let method = DeleteProjectSvc(inner);
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
                "/permio.organisation.v1.OrganisationService/GetPrincipalUsageStats" => {
                    #[allow(non_camel_case_types)]
                    struct GetPrincipalUsageStatsSvc<T: OrganisationService>(pub Arc<T>);
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::GetPrincipalUsageStatsRequest>
                    for GetPrincipalUsageStatsSvc<T> {
                        type Response = super::GetPrincipalUsageStatsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetPrincipalUsageStatsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::get_principal_usage_stats(
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
                        let method = GetPrincipalUsageStatsSvc(inner);
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
                "/permio.organisation.v1.OrganisationService/GetUsageHistory" => {
                    #[allow(non_camel_case_types)]
                    struct GetUsageHistorySvc<T: OrganisationService>(pub Arc<T>);
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::GetUsageHistoryRequest>
                    for GetUsageHistorySvc<T> {
                        type Response = super::GetUsageHistoryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetUsageHistoryRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::get_usage_history(
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
                        let method = GetUsageHistorySvc(inner);
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
                "/permio.organisation.v1.OrganisationService/GetOrganisationDashboard" => {
                    #[allow(non_camel_case_types)]
                    struct GetOrganisationDashboardSvc<T: OrganisationService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<super::GetOrganisationDashboardRequest>
                    for GetOrganisationDashboardSvc<T> {
                        type Response = super::GetOrganisationDashboardResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GetOrganisationDashboardRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::get_organisation_dashboard(
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
                        let method = GetOrganisationDashboardSvc(inner);
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
                "/permio.organisation.v1.OrganisationService/RemoveUserFromOrganisation" => {
                    #[allow(non_camel_case_types)]
                    struct RemoveUserFromOrganisationSvc<T: OrganisationService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: OrganisationService,
                    > tonic::server::UnaryService<
                        super::RemoveUserFromOrganisationRequest,
                    > for RemoveUserFromOrganisationSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::RemoveUserFromOrganisationRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as OrganisationService>::remove_user_from_organisation(
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
                        let method = RemoveUserFromOrganisationSvc(inner);
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
    impl<T: OrganisationService> Clone for OrganisationServiceServer<T> {
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
    impl<T: OrganisationService> tonic::server::NamedService
    for OrganisationServiceServer<T> {
        const NAME: &'static str = "permio.organisation.v1.OrganisationService";
    }
}
