use warp::Filter;

use anyhow::Result;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use std::collections::HashMap;

use std::path::PathBuf;
use structopt::StructOpt;

use serde::{Deserialize, Serialize};

/// Health struct returned by the RPC
#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, StructOpt)]
#[structopt(
    name = "node-health-check",
    about = "Sidecar for substrate node to report health compatible with AWS ALB."
)]
struct Opt {
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

async fn poller(opts: Opt) {
    loop {
        // poll the services here
        //
        let resp = async {
            reqwest::get(&opts.node_rpc_endpoint)
                .await?
                .json::<Health>()
                .await
        };

        println!("{:#?}", resp.await);

        tokio::time::sleep(Duration::from_secs(opts.poll_interval)).await;
    }
}

#[tokio::main]
async fn main() {
    let opts = Opt::from_args();

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let health = warp::path!("health").map(|| format!("healthy!"));

    tokio::spawn(poller(opts));

    warp::serve(health).run(([0, 0, 0, 0], 9955)).await;
}
