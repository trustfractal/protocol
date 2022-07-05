use actix_web::{error::*, *};
use ramhorns::Ramhorns;

pub fn resources() -> impl Iterator<Item = Resource> {
    vec![web::resource("/swap_chains").to(index)].into_iter()
}

async fn index(templates: web::Data<Ramhorns>) -> actix_web::Result<HttpResponse> {
    let page = templates
        .get("swap_chains/index.html")
        .ok_or_else(|| ErrorInternalServerError("Could not find template"))?
        .source();
    crate::pages::html_page(templates.clone(), page)
}
