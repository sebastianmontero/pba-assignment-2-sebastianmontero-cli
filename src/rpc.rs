use crate::error::Error;
use jsonrpsee_core::client::{ClientT, Subscription, SubscriptionClientT};
use jsonrpsee_core::rpc_params;
use jsonrpsee_ws_client::{WsClient, WsClientBuilder};
use std::collections::HashMap;

pub struct RPC {
    client: WsClient,
}

impl RPC {
    pub async fn new(addr: &str) -> Result<RPC, Error> {
        // let addr = "localhost:9944";
        let url = format!("ws://{}", addr);
        let client = WsClientBuilder::default().build(&url).await?;
        Ok(RPC { client })
    }

    pub async fn request(&self, method: &str, param: &str) -> Result<Option<String>, Error> {
        // println!("Making rpc call, method:{}, param:{}", method, param);
        let response: Option<String> = self.client.request(method, rpc_params![param]).await?;
        // println!("response: {:?}", response);
        Ok(response)
    }

    pub async fn subscribe(
        &self,
        method: &str,
        param: &str,
    ) -> Result<Subscription<HashMap<String, String>>, Error> {
        // println!("Subscribing to, method:{}, param:{}", method, param);
        let subs = self
            .client
            .subscribe(method, rpc_params![param], "")
            .await?;
        // println!("response: {:?}\n\n\n", subs);
        Ok(subs)
    }
}
