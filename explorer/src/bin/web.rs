use actix_web::*;
use fractal_explorer::pages;
use structopt::StructOpt;

#[derive(StructOpt, Clone)]
struct Options {
    #[structopt(long, default_value = "8080")]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let options = Options::from_args();
    let port = options.port;

    HttpServer::new(move || App::new().data(options.clone()).service(pages::service()))
        .bind(("0.0.0.0", port))?
        .run()
        .await
}
