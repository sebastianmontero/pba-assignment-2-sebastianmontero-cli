use jsonrpsee_core::client::{ClientT};
use jsonrpsee_core::rpc_params;
use jsonrpsee_ws_client::{WsClient,WsClientBuilder};
use crate::error::Error;

pub struct RPC {
  client: WsClient,
}

impl RPC {

  pub async fn new(addr: &str) -> Result<RPC, Error> {
    // let addr = "localhost:9944";
    let url = format!("ws://{}", addr);
    let client = WsClientBuilder::default().build(&url).await?;
    Ok(
      RPC { client }
    )
  }

  pub async fn request(&self, method: &str, param: &str)-> Result<Option<String>, Error>{
    println!("Making rpc call, method:{}, param:{}", method, param);
    let response: Option<String> = self.client.request(method, rpc_params![param]).await?;
    println!("response: {:?}", response);
    Ok(response)
  }
}

