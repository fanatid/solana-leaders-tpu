use {
    anyhow::Context, clap::Parser, solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::commitment_config::CommitmentConfig, std::collections::HashMap,
};

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about)]
struct Args {
    /// Service endpoint
    #[clap(short, long, default_value_t = String::from("http://127.0.0.1:8899"))]
    endpoint: String,
}

#[derive(Debug)]
struct ScheduleInfo {
    identity: String,
    slots: Vec<usize>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let client = RpcClient::new_with_commitment(args.endpoint, CommitmentConfig::finalized());

    let slot = client.get_slot().await.context("failed to fetch slot")?;
    println!("finalized slot: {slot}");

    let schedule = client
        .get_leader_schedule(Some(slot))
        .await
        .context("failed to fetch schedule {slot}")?
        .ok_or_else(|| anyhow::anyhow!("empty schedule for slot {slot}"))?;
    let mut schedule = schedule
        .into_iter()
        .map(|(identity, slots)| ScheduleInfo { identity, slots })
        .collect::<Vec<_>>();
    schedule.sort_by(|a, b| {
        a.slots
            .len()
            .cmp(&b.slots.len())
            .then_with(|| a.identity.cmp(&b.identity))
            .reverse()
    });

    let cluster_nodes = client
        .get_cluster_nodes()
        .await
        .context("failed to fetch cluster nodes")?
        .into_iter()
        .map(|node| (node.pubkey.clone(), node))
        .collect::<HashMap<_, _>>();

    println!(
        "                                            Pubkey  Slots                    TPU  Found"
    );
    for info in schedule {
        let node = cluster_nodes.get(&info.identity);
        println!(
            "{:>50} {:>6} {:>22} {:>6}",
            info.identity,
            info.slots.len(),
            node.and_then(|node| node.tpu_quic)
                .map(|tpu| format!("{tpu:?}"))
                .unwrap_or_else(|| "not found".to_owned()),
            node.is_some()
        );
    }

    Ok(())
}
