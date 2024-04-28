use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Instant,
};

use tokio::sync::{watch, RwLock};
use tracing::error;
use veloren_query_server::{
    client::QueryClient,
    proto::{ServerBattleMode, ServerInfo},
    server::{Metrics, QueryServer},
};

const DEFAULT_SERVER_INFO: ServerInfo = ServerInfo {
    git_hash: ['\0'; 8],
    players_count: 100,
    player_cap: 300,
    battlemode: ServerBattleMode::GlobalPvE,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 14006);
    let (_sender, receiver) = watch::channel(DEFAULT_SERVER_INFO);
    let mut server = QueryServer::new(addr, receiver);
    let metrics = Arc::new(RwLock::new(Metrics::default()));
    let metrics2 = Arc::clone(&metrics);

    tokio::task::spawn(async move { server.run(metrics2).await.unwrap() });

    let mut client = QueryClient::new(addr);
    let (info, ping) = client.server_info().await.unwrap();

    println!("Ping = {}ms", ping.as_millis());
    println!("Server info: {info:?}");
    assert_eq!(info, DEFAULT_SERVER_INFO);

    let start = Instant::now();

    for _i in 0..32 {
        if let Err(error) = client.server_info().await {
            error!(?error, "Server info request error");
        }
    }

    println!("Metrics = {:#?}", metrics.read().await);
    dbg!(start.elapsed());
}
