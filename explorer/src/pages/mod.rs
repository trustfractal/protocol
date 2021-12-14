use actix_web::{error::*, *};
use block_pool::Pool;
use ramhorns::{Content, Ramhorns};
use std::collections::BTreeMap;

use crate::data::*;

pub fn resources() -> Vec<Resource> {
    vec![
        web::resource("/").to(home),
        web::resource("/metrics/identities").to(metrics_identities),
    ]
}

pub fn templates() -> anyhow::Result<Ramhorns> {
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
) -> actix_web::Result<HttpResponse> {
    let page = Page {
        page: content.to_string(),
    };

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            templates
                .get("root.html")
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
        .get("metrics/identities.html")
        .ok_or_else(|| ErrorInternalServerError("Could not find template"))?
        .render(&counts);
    Ok(html_page(templates, page)?)
}

async fn retry_blocking<R, E, F>(f: F) -> Result<R, E>
where
    F: FnMut() -> Result<R, E> + Send + 'static,
    R: Send + 'static,
    E: Send + core::fmt::Debug + 'static,
{
    let mut tries = 3;
    let f = std::sync::Arc::new(std::sync::Mutex::new(f));
    loop {
        let f = std::sync::Arc::clone(&f);
        let result = web::block(move || {
            let mut f = f.lock().unwrap();
            f()
        })
        .await;

        match result {
            Ok(v) => return Ok(v),
            Err(BlockingError::Error(e)) => return Err(e),
            Err(BlockingError::Canceled) => {
                tries -= 1;
                if tries == 0 {
                    panic!("Blocking call cancelled.");
                } else {
                    log::warn!("Blocking call cancelled.");
                }
            }
        }
    }
}

pub async fn not_found() -> actix_web::Result<String> {
    Err(error::ErrorNotFound("Not Found"))
}

#[derive(Content)]
struct HomeData {
    blocks: Vec<Block>,
    extrinsics: Vec<ExtrinsicNoJson>,
}

async fn home(
    templates: web::Data<Ramhorns>,
    pg: web::Data<Pool<postgres::Client>>,
) -> actix_web::Result<HttpResponse> {
    let home_data = retry_blocking(move || get_home_data(&mut pg.take()))
        .await
        .map_err(ErrorInternalServerError)?;

    let page = templates
        .get("home.html")
        .ok_or_else(|| ErrorInternalServerError("Could not find template"))?
        .render(&home_data);
    Ok(html_page(templates, page)?)
}

fn get_home_data(pg: &mut postgres::Client) -> anyhow::Result<HomeData> {
    let blocks = pg
        .query(
            "SELECT json FROM block_json ORDER BY number DESC LIMIT 20",
            &[],
        )?
        .into_iter()
        .map(|row| serde_json::from_str(row.get(&"json")))
        .collect::<Result<_, _>>()?;

    let extrinsics = pg
        .query(
            "SELECT json FROM extrinsic_json
            WHERE
                CAST(json AS json)->>'section' != 'timestamp' AND
                CAST(json AS json)->>'success' = 'true'
            ORDER BY block_number DESC, index LIMIT 20",
            &[],
        )?
        .into_iter()
        .map(|row| serde_json::from_str(row.get(&"json")).map(|e: Extrinsic| e.without_json))
        .collect::<Result<_, _>>()?;

    Ok(HomeData { blocks, extrinsics })
}
