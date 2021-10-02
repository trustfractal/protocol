use warp::Filter;

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let health = warp::path!("health").map(|| format!("healthy!"));

    warp::serve(health).run(([0, 0, 0, 0], 9955)).await;
}
