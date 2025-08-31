use subxt::{Config, OnlineClient, blocks::ExtrinsicDetails};
use tokio::runtime::Runtime;

mod our_rpc_client;

pub type AssetHubConfig = subxt::SubstrateConfig;
pub const ASSETHUB_NETWORK_INDEX: u8 = 12;
pub const BLOCK_NUMBER: u128 = 9520865;

#[subxt::subxt(runtime_metadata_path = "./assethub.metadata.binary.scale")]
pub mod assethub {}

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(get_block_events());
}

async fn get_block_events() {
    let (online_client, legacy_rpc_client) =
        our_rpc_client::build_base_client::<AssetHubConfig>().await;
    let hash = legacy_rpc_client
        .chain_get_block_hash(Some(BLOCK_NUMBER.into()))
        .await
        .unwrap()
        .unwrap();
    let block = online_client.blocks().at(hash).await.unwrap();
    let extrinsics = block.extrinsics().await.unwrap();
    let extrinsics = extrinsics
        .iter()
        .filter(|ext| filter_extrinsic(ext))
        .collect::<Vec<_>>();
    let extrinsic = extrinsics.first().unwrap();
    let events = extrinsic.events().await.unwrap();
    for event in events.all_events_in_block().iter() {
        if let Ok(details) = event.as_ref() {
            println!("Event {}: {:?}", details.pallet_name(), details.variant_name());
        }
        else {
            print!("Error parsing event");
        }
    }
}

fn filter_extrinsic<ConfigT: Config>(
    extrinsic: &ExtrinsicDetails<ConfigT, OnlineClient<ConfigT>>,
) -> bool {
    extrinsic
        .as_extrinsic::<assethub::proxy::calls::types::Proxy>()
        .is_ok()
}
