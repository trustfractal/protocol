use actix_web::{error::BlockingError, *};
use block_pool::Pool;
use derive_more::*;
use ramhorns::{Content, Ramhorns};

pub fn service() -> Scope {
    web::scope("/")
        .data(templates().unwrap())
        .service(web::resource("/metrics/identities").to(metrics_identities))
}

#[derive(Debug, Display, Error)]
struct Error {
    error: anyhow::Error,
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error { error }
    }
}

impl error::ResponseError for Error {}

fn templates() -> anyhow::Result<Ramhorns> {
    let mod_file = std::path::PathBuf::from(file!());
    let pages = mod_file.parent().unwrap();
    Ok(Ramhorns::from_folder(pages)?)
}

#[derive(Content)]
struct Page {
    page: String,
}

fn html_page(
    templates: web::Data<Ramhorns>,
    content: impl ToString,
) -> anyhow::Result<HttpResponse> {
    let page = Page {
        page: content.to_string(),
    };

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(templates.get("root.html").unwrap().render(&page)))
}

#[derive(Content)]
struct IdentityCounts {
    points: Vec<Point>,
}

#[derive(Content)]
struct Point {
    x: f64,
    y: f64,
}

async fn metrics_identities(
    templates: web::Data<Ramhorns>,
    pg: web::Data<Pool<postgres::Client>>,
) -> Result<HttpResponse, Error> {
    let values = loop {
        let pg = pg.clone();
        let result = web::block(move || {
            let mut pg = pg.take();
            crate::indexing::identities::get_counts(1000, &mut pg)
        })
        .await;
        match result {
            Ok(v) => break v,
            Err(BlockingError::Canceled) => continue,
            Err(BlockingError::Error(e)) => return Err(e)?,
        }
    };

    let counts = IdentityCounts {
        points: values
            .into_iter()
            .map(|(x, y)| Point {
                x: x as f64,
                y: y as f64,
            })
            .collect(),
    };

    let page = templates
        .get("metrics/identities.html")
        .ok_or(anyhow::anyhow!("Could not find template"))?
        .render(&counts);
    Ok(html_page(templates, page)?)
}
