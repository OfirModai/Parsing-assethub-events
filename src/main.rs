use subxt::{Config, OnlineClient};
use tokio::runtime::Runtime;
use subxt::SubstrateConfig;
use subxt::config::Hasher;
use parity_scale_codec::Decode;

pub type AssetHubConfig = subxt::SubstrateConfig;

#[subxt::subxt(runtime_metadata_path = "./assethub.metadata.binary.scale")]
pub mod assethub {}

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(get_block_events());
}


async fn get_block_events() {
    let client = OnlineClient::<AssetHubConfig>::from_url("wss://polkadot-asset-hub-rpc.polkadot.io")
        .await
        .expect("Failed to connect to node");
    let bytes = hex::decode(&"8cec6507f89ccb7e156075f167b979d69d79ed6f2cd453f8f221319484da8960").unwrap();
    let hash = <<SubstrateConfig as Config>::Hasher as Hasher>::Output::decode(&mut &bytes[..]).unwrap();
    let events = client.events().at(hash).await.unwrap();
    for (index, event) in events.iter().enumerate() {
        println!("parsing event #{}", index);
        if let Ok(details) = event.as_ref() {
            println!("Event {}: {:?}", details.pallet_name(), details.variant_name());
        }
        else {
            print!("Error parsing event");
        }
    }
}