use actix_web::{error::*, *};
use block_pool::Pool;
use ramhorns::{Content, Ramhorns};
use std::collections::BTreeMap;

use crate::retry_blocking;

mod entities;
mod home;

pub fn resources() -> impl Iterator<Item = Resource> {
    vec![
        web::resource("/").to(home::home),
        web::resource("/metrics/identities").to(metrics_identities),
    ]
    .into_iter()
    .chain(crate::swap_chains::resources())
    .chain(entities::resources())
}

pub fn templates() -> anyhow::Result<Ramhorns> {
    Ok(Ramhorns::from_folder("explorer/src")?)
}

#[derive(Content)]
struct Page {
    page: String,
}

pub fn html_page(
    templates: web::Data<Ramhorns>,
    content: impl ToString,
) -> actix_web::Result<HttpResponse> {
    let page = Page {
        page: content.to_string(),
    };

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            templates
                .get("pages/root.html")
                .ok_or_else(|| ErrorInternalServerError("Could not find template"))?
                .render(&page),
        ))
}

#[derive(Content)]
struct IdentityCounts {
    points: Vec<Point>,
    latest: Latest,
    deltas: Vec<Delta>,
}

#[derive(Content)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Content, Default)]
struct Latest {
    block: u64,
    count: u64,
}

#[derive(Content)]
struct Delta {
    name: String,
    amount: u64,
    percent: f32,
}

impl Delta {
    fn from_diff_values(block_diff: u64, name: &str, values: &BTreeMap<u64, u64>) -> Option<Self> {
        let (max_block, max_count) = values.range(..).next_back()?;
        let prev_block = max_block.checked_sub(block_diff)?;
        let (_, prev_count) = values.range(..=prev_block).next_back()?;
        if *prev_count == 0 {
            return None;
        }

        Some(Delta {
            name: name.to_string(),
            amount: max_count - prev_count,
            percent: ((*max_count as f32 / *prev_count as f32) - 1.) * 100.,
        })
    }
}

async fn metrics_identities(
    templates: web::Data<Ramhorns>,
    pg: web::Data<Pool<postgres::Client>>,
) -> actix_web::Result<HttpResponse> {
    let deltas = {
        const DAY: u64 = 14400;

        let mut map = BTreeMap::new();
        map.insert(DAY, "Day");
        map.insert(DAY * 7, "Week");
        map.insert(DAY * 91, "Quarter");
        map.insert(DAY * 365, "Year");

        map
    };

    let include_block_deltas = deltas.keys().cloned().collect::<Vec<_>>();
    let values = retry_blocking(move || {
        let mut pg = pg.take();
        let include = include_block_deltas.clone();
        crate::indexing::identities::get_counts(1000, include, &mut pg)
    })
    .await
    .map_err(ErrorInternalServerError)?;

    let deltas = deltas
        .into_iter()
        .filter_map(|(diff, name)| Delta::from_diff_values(diff, name, &values))
        .collect();
    let latest = values
        .range(..)
        .next_back()
        .map(|(&block, &count)| Latest { block, count })
        .unwrap_or_default();
    let counts = IdentityCounts {
        points: values
            .into_iter()
            .map(|(x, y)| Point {
                x: x as f32,
                y: y as f32,
            })
            .collect(),
        latest,
        deltas,
    };

    let page = templates
        .get("pages/metrics/identities.html")
        .ok_or_else(|| ErrorInternalServerError("Could not find template"))?
        .render(&counts);
    html_page(templates, page)
}

pub async fn not_found() -> actix_web::Result<String> {
    Err(error::ErrorNotFound("Not Found"))
}
