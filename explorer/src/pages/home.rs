use actix_web::{error::*, *};
use block_pool::Pool;
use ramhorns::*;
use serde::*;
use std::collections::HashSet;

use crate::{
    data::*,
    pages::{html_page, retry_blocking},
};

#[derive(Content)]
struct HomeData {
    blocks: Vec<Block>,
    extrinsics: Vec<ExtrinsicNoJson>,
    active_filters: Vec<Filter>,
    potential_filters: Vec<Filter>,
}

#[derive(Content)]
struct Filter {
    url: String,
    text: String,
}

impl HomeData {
    fn build(blocks: Vec<Block>, extrinsics: Vec<Extrinsic>, filters: &ActiveFilters) -> Self {
        HomeData {
            blocks,
            potential_filters: Self::potential_filters(&extrinsics, filters),
            active_filters: Self::active_filters(filters),
            extrinsics: extrinsics.into_iter().map(|e| e.without_json).collect(),
        }
    }

    fn potential_filters(extrinsics: &[Extrinsic], filters: &ActiveFilters) -> Vec<Filter> {
        extrinsics
            .iter()
            .map(|e| e.method.to_string())
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|s| Filter {
                url: Self::url(filters.iter().chain(std::iter::once(s.as_ref()))),
                text: s,
            })
            .collect()
    }

    fn url<'a>(extrs: impl Iterator<Item = &'a str>) -> String {
        let filters = extrs.collect::<Vec<_>>().join(",");
        if filters.is_empty() {
            ".".to_string()
        } else {
            format!("?hide_extrinsics={}", filters)
        }
    }

    fn active_filters(filters: &ActiveFilters) -> Vec<Filter> {
        filters
            .iter()
            .map(|active| Filter {
                url: Self::url(filters.iter().filter(|&f| f != active)),
                text: active.to_string(),
            })
            .collect()
    }
}

#[derive(Deserialize, Debug)]
pub struct ActiveFilters {
    hide_extrinsics: Option<String>,
}

impl ActiveFilters {
    fn iter(&self) -> impl Iterator<Item = &str> {
        self.hide_extrinsics
            .iter()
            .flat_map(|s| s.split(','))
            .filter(|s| !s.is_empty())
    }
}

pub async fn home(
    templates: web::Data<Ramhorns>,
    pg: web::Data<Pool<postgres::Client>>,
    filters: web::Query<ActiveFilters>,
) -> actix_web::Result<HttpResponse> {
    let home_data = retry_blocking(move || get_home_data(&mut pg.take(), &filters))
        .await
        .map_err(ErrorInternalServerError)?;

    let page = templates
        .get("home.html")
        .ok_or_else(|| ErrorInternalServerError("Could not find template"))?
        .render(&home_data);
    html_page(templates, page)
}

fn get_home_data(pg: &mut postgres::Client, filters: &ActiveFilters) -> anyhow::Result<HomeData> {
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
                CAST(json AS json)->>'success' = 'true' AND
                NOT (CAST(json AS json)->>'method' = ANY($1))
            ORDER BY block_number DESC, index LIMIT 20",
            &[&filters.iter().collect::<Vec<&str>>()],
        )?
        .into_iter()
        .map(|row| serde_json::from_str(row.get(&"json")))
        .collect::<Result<_, _>>()?;

    Ok(HomeData::build(blocks, extrinsics, filters))
}
