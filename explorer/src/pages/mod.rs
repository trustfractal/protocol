use actix_web::*;
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

async fn metrics_identities(templates: web::Data<Ramhorns>) -> Result<HttpResponse, Error> {
    let counts = IdentityCounts {
        points: vec![
            Point { x: 3., y: 5. },
            Point { x: 10., y: 8. },
        ],
    };

    let page = templates
        .get("metrics/identities.html")
        .ok_or(anyhow::anyhow!("Could not find template"))?
        .render(&counts);
    Ok(html_page(templates, page)?)
}
