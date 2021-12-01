use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use structopt::*;

#[derive(StructOpt)]
struct Options {
    #[structopt(long, default_value = "8080")]
    port: u16,
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let options = Options::from_args();

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
    .bind(("0.0.0.0", options.port))?
    .run()
    .await
}
