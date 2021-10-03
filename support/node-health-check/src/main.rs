use std::convert::Infallible;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::{sync::Arc, time::Duration};

use anyhow::Result;
use reqwest::StatusCode;
use serde::Deserialize;
use structopt::StructOpt;
use warp::Filter;

/// Health struct returned by the RPC
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    /// Number of connected peers
    pub peers: usize,
    /// Is the node syncing
    pub is_syncing: bool,
    /// Should this node have any peers
    /// Might be false for local chains or when running without discovery.
    pub should_have_peers: bool,
}

impl Health {
    fn is_healthy(&self, required_peers: usize) -> bool {
        if self.is_syncing {
            return false;
        }
        if self.should_have_peers {
            return self.peers >= required_peers;
        }
        true
    }
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "node-health-check",
    about = "Sidecar for substrate node to report health compatible with AWS ALB."
)]
struct Opt {
    /// Debug
    #[structopt(short, long)]
    debug: bool,

    /// Set poll_interval
    #[structopt(short = "i", default_value = "5")]
    poll_interval_in_secs: u64,

    /// Set minimum peers to be connected for this node for it to be considered healthy
    #[structopt(default_value = "2")]
    min_peers: usize,

    /// Node rpc endpoint
    #[structopt(short, long, env, default_value = "http://localhost:9933/health")]
    node_rpc_endpoint: String,

    // The port to listen on
    #[structopt(short, long, default_value = "9955")]
    port: u16,
}

async fn poller(opts: Opt, healthy: Arc<AtomicBool>) {
    loop {
        let resp = async {
            reqwest::get(&opts.node_rpc_endpoint)
                .await?
                .json::<Health>()
                .await
        };

        match resp.await {
            Ok(health) => {
                if opts.debug {
                    eprintln!("Response = {:?}", health);
                    eprintln!("Healthy  = {:?}", health.is_healthy(opts.min_peers));
                }
                healthy.store(health.is_healthy(opts.min_peers), Ordering::Relaxed);
            }
            Err(error) => {
                eprintln!("{:?}", error);
                healthy.store(false, Ordering::Relaxed);
            }
        }

        tokio::time::sleep(Duration::from_secs(opts.poll_interval_in_secs)).await;
    }
}

async fn return_health_status(healthy: Arc<AtomicBool>) -> Result<impl warp::Reply, Infallible> {
    if healthy.load(Ordering::Relaxed) {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::SERVICE_UNAVAILABLE)
    }
}

#[tokio::main]
async fn main() {
    let opts = Opt::from_args();
    let healthy = Arc::new(AtomicBool::new(true));

    let health_check = {
        let healthy = healthy.clone();
        warp::path!("health")
            .map(move || healthy.clone())
            .and_then(return_health_status)
    };

    tokio::spawn(warp::serve(health_check).run(([0, 0, 0, 0], opts.port)));
    poller(opts, healthy).await;
}
