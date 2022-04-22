use actix_web::*;
use fractal_explorer::{indexing, pages};
use structopt::StructOpt;

#[derive(StructOpt, Clone)]
struct Options {
    #[structopt(long, default_value = "8080")]
    port: u16,

    #[structopt(long)]
    postgres: String,

    #[structopt(long)]
    redirect_https: bool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    ::simple_logger::init_with_level(log::Level::Info).unwrap();

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

        let mut app = App::new()
            .data(pool)
            .data(options.clone())
            .data(pages::templates().unwrap())
            .wrap(
                actix_web_middleware_redirect_scheme::RedirectSchemeBuilder::new()
                    .enable(options.redirect_https)
                    .build(),
            );

        app = pages::resources()
            .into_iter()
            .fold(app, |a, resource| a.service(resource));

        app.service(indexing::id_to_entity::redirect_id)
            .service(web::resource("*").route(web::get().to(pages::not_found)))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
