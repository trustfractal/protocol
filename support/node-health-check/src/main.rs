use warp::Filter;

use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "node-health-check",
    about = "Sidecar for substrate node to report health compatible with AWS ALB."
)]
struct Opt {
    /// Set poll_interval
    #[structopt(short = "i", default_value = "5")]
    poll_interval: f64,

    /// Node rpc endpoint
    #[structopt(short, long, env, default_value = "http://localhost:9933")]
    node_rpc_endpoint: String,

    // The port to listen on
    #[structopt(short, long, default_value = "9955")]
    port: u16,
}

async fn poller() {
    loop {
        // poll the services here
        //

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let health = warp::path!("health").map(|| format!("healthy!"));

    warp::serve(health).run(([0, 0, 0, 0], 9955)).await;
}
