use subxt::{
    Config, OnlineClient,
    backend::{
        legacy::LegacyRpcMethods,
        rpc::{RawRpcFuture, RawRpcSubscription, RpcClient, RpcClientT},
    },
};

use jsonrpsee_core::{client::ClientT, traits::ToRpcParams};
use jsonrpsee_http_client::{HeaderMap, HttpClient};

use serde_json::value::RawValue;
use tower::limit::ConcurrencyLimit;

type JsonRpseeHttpClientType = HttpClient<
    ConcurrencyLimit<
        tower_http::decompression::Decompression<jsonrpsee_http_client::transport::HttpBackend>,
    >,
>;

pub(crate) struct SubxtHttpClient(JsonRpseeHttpClientType);
pub(crate) const DEFAULT_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);
const MAX_CONCURRENT_REQUESTS: usize = 1024;

struct Params(Option<Box<RawValue>>);

impl ToRpcParams for Params {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
        Ok(self.0)
    }
}

pub const NODE_URL: &str = ""; // enter your node url here
pub const TOKEN: &str = ""; // enter your node token here

impl SubxtHttpClient {
    pub(crate) fn new(url: impl AsRef<str>, headers: Option<HeaderMap>) -> Self {
        let mut builder = get_json_rpsee_client_builder();
        if let Some(headers) = headers {
            builder = builder.set_headers(headers);
        }
        let client = builder.build(url).unwrap();
        SubxtHttpClient(client)
    }
}

type JsonRpseeHttpClientBuilder = jsonrpsee_http_client::HttpClientBuilder<
    tower::layer::util::Stack<
        tower_http::decompression::DecompressionLayer,
        tower::layer::util::Stack<
            tower::limit::ConcurrencyLimitLayer,
            tower::layer::util::Identity,
        >,
    >,
>;

impl RpcClientT for SubxtHttpClient {
    fn request_raw<'a>(
        &'a self,
        method: &'a str,
        params: Option<Box<RawValue>>,
    ) -> RawRpcFuture<'a, Box<RawValue>> {
        Box::pin(async move {
            let res = self.0.request(method, Params(params)).await.map_err(
                |e: jsonrpsee_core::ClientError| subxt::ext::subxt_rpcs::Error::Client(Box::new(e)),
            )?;
            Ok(res)
        })
    }

    fn subscribe_raw<'a>(
        &'a self,
        _sub: &'a str,
        _params: Option<Box<RawValue>>,
        _unsub: &'a str,
    ) -> RawRpcFuture<'a, RawRpcSubscription> {
        panic!("HTTP Client does not support subscription");
    }
}

fn get_json_rpsee_client_builder() -> JsonRpseeHttpClientBuilder {
    let middleware_stack = tower::ServiceBuilder::new()
        .concurrency_limit(MAX_CONCURRENT_REQUESTS)
        .layer(
            tower_http::decompression::DecompressionLayer::new()
                .gzip(true)
                .deflate(true),
        );
    jsonrpsee_http_client::HttpClientBuilder::default()
        .set_http_middleware(middleware_stack)
        .request_timeout(DEFAULT_REQUEST_TIMEOUT)
}

pub(crate) async fn build_base_client<ConfigT: Config>()
-> (OnlineClient<ConfigT>, LegacyRpcMethods<ConfigT>) {
    let url = NODE_URL;
    let mut headers = jsonrpsee_http_client::HeaderMap::new();
    let header = reqwest::header::HeaderName::from_static("x-api-key");
    let value = reqwest::header::HeaderValue::from_str(TOKEN).unwrap();

    let header_name = header.to_string();
    let header_value = value.to_str().unwrap().as_bytes().to_owned();

    let header_name =
        http_v1_for_jsonrpsee::header::HeaderName::from_bytes(header_name.as_bytes()).unwrap();
    let header_value =
        http_v1_for_jsonrpsee::header::HeaderValue::from_bytes(&header_value).unwrap();

    headers.insert(header_name, header_value);
    let subxt_http_client = SubxtHttpClient::new(url, Some(headers));
    let rpc_client = RpcClient::new(subxt_http_client);
    let legacy_rpc_client = LegacyRpcMethods::<ConfigT>::new(rpc_client.clone());
    let client = OnlineClient::<ConfigT>::from_rpc_client(rpc_client)
        .await
        .expect("Failed to connect to node");

    (client, legacy_rpc_client)
}
