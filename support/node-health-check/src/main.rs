use reqwest::StatusCode;
use warp::Filter;

use anyhow::Result;
use std::convert::Infallible;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use warp::header;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use std::collections::HashMap;

use std::path::PathBuf;
use structopt::StructOpt;

use serde::{Deserialize, Serialize};

/// Health struct returned by the RPC
#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    /// Number of connected peers
    pub peers: usize,
    /// Is the node syncing
    pub is_syncing: bool,
    /// Should this node have any peers
    ///
    /// Might be false for local chains or when running without discovery.
    pub should_have_peers: bool,
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
    poll_interval: u64,

    /// Node rpc endpoint
    #[structopt(short, long, env, default_value = "http://localhost:9933/health")]
    node_rpc_endpoint: String,

    // The port to listen on
    #[structopt(short, long, default_value = "9955")]
    port: u16,
}

async fn poller(opts: Opt, healthy: Arc<AtomicBool>) {
    loop {
        // poll the services here
        let resp = async {
            reqwest::get(&opts.node_rpc_endpoint)
                .await?
                .json::<Health>()
                .await
        };

        match resp.await {
            Ok(health) => {
                // we consider this node healthy when its
                // connected to peers and its not synchronizing
                let node_is_healthy =
                    !health.is_syncing && health.peers >= 2 && health.should_have_peers;

                if opts.debug {
                    eprintln!("Response = {:?}", health);
                    eprintln!("Healthy  = {:?}", node_is_healthy);
                }
                healthy.store(node_is_healthy, Ordering::Relaxed);
            }
            Err(error) => {
                eprint!("{:?}", error);
                healthy.store(false, Ordering::Relaxed);
            }
        }

        tokio::time::sleep(Duration::from_secs(opts.poll_interval)).await;
    }
}

async fn return_health_status(healthy: Arc<AtomicBool>) -> Result<impl warp::Reply, Infallible> {
    if healthy.load(Ordering::Relaxed) {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::NOT_FOUND)
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

    // start background poller
    tokio::spawn(poller(opts.clone(), healthy));

    // server the health_check
    warp::serve(health_check)
        .run(([0, 0, 0, 0], opts.port))
        .await;
}
