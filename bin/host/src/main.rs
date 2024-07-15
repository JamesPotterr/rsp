use alloy_provider::ReqwestProvider;
use clap::Parser;
use rsp_host_executor::HostExecutor;
use sp1_sdk::{ProverClient, SP1Stdin};
use tracing_subscriber::{
    filter::EnvFilter, fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
};

/// The arguments for the host executable
#[derive(Debug, Clone, Parser)]
struct HostArgs {
    /// The block number of the block to execute.
    #[clap(long)]
    block_number: u64,
    /// The rpc url used to fetch data about the block.
    #[clap(long)]
    rpc_url: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Initialize the logger.
    tracing_subscriber::registry().with(fmt::layer()).with(EnvFilter::from_default_env()).init();

    // Parse the command line arguments.
    let args = HostArgs::parse();

    // Setup the provider.
    let provider = ReqwestProvider::new_http(args.rpc_url.parse()?);

    // Setup the host executor.
    let host_executor = HostExecutor::new(provider);

    // Execute the host.
    let guest_input =
        host_executor.execute(args.block_number).await.expect("failed to execute host");

    // Generate the proof.
    let client = ProverClient::new();

    // Setup the proving key and verification key.
    let (pk, _) = client.setup(include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf"));

    // Execute the block inside the zkVM.
    let mut stdin = SP1Stdin::new();
    let buffer = serde_json::to_vec(&guest_input).unwrap();
    stdin.write_vec(buffer);
    client.execute(&pk.elf, stdin).unwrap();

    // // Generate the proof.
    // let mut stdin = SP1Stdin::new();
    // stdin.write(&guest_input);
    // let proof = client.prove(&pk, stdin).unwrap();

    // // Verify the proof.
    // client.verify(&proof, &vk).unwrap();

    Ok(())
}