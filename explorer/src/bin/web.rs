use actix_web::*;
use fractal_explorer::pages;
use structopt::StructOpt;

#[derive(StructOpt, Clone)]
struct Options {
    #[structopt(long, default_value = "8080")]
    port: u16,

    #[structopt(long)]
    postgres: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let options = Options::from_args();
    let port = options.port;

    HttpServer::new(move || {
        let pool = block_pool::Pool::new(
            std::iter::repeat(())
                .map(|_| fractal_explorer::postgres::connect(&options.postgres))
                .take(10)
                .collect::<anyhow::Result<Vec<_>>>()
                .unwrap(),
        );

        App::new()
            .data(pool)
            .data(options.clone())
            .service(pages::service())
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
