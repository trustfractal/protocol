use actix_web::{error::*, *};
use block_pool::Pool;
use ramhorns::Ramhorns;
use std::{sync::Arc, time::SystemTime};

use crate::{
    data::ExtrinsicNoJson,
    indexing::{
        address_extrinsics,
        block_timestamps::{self, BlockId},
    },
    pages::{html_page, retry_blocking},
};

pub fn resources() -> impl Iterator<Item = Resource> {
    vec![web::resource("/address/{address}").to(address)].into_iter()
}

#[derive(ramhorns::Content)]
struct AddressContent {
    address: String,
    extrinsics: Vec<ExtrinsicContent>,
}

#[derive(ramhorns::Content)]
struct ExtrinsicContent {
    extr: ExtrinsicNoJson,
    timestamp: Option<u64>,
}

async fn address(
    templates: web::Data<Ramhorns>,
    pg: web::Data<Pool<postgres::Client>>,
    web::Path(address): web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let pg = Arc::new(pg);
    let addr_ref = address.clone();

    let extrinsics = {
        let pg = Arc::clone(&pg);
        retry_blocking(move || address_extrinsics::for_address(&addr_ref, &mut pg.take()))
            .await
            .map_err(ErrorInternalServerError)?
    };

    let blocks = extrinsics
        .iter()
        .map(|e| BlockId::Hash(e.block.clone()))
        .collect::<Vec<_>>();
    let block_timestamps =
        retry_blocking(move || block_timestamps::get(blocks.iter().cloned(), &mut pg.take()))
            .await
            .map_err(ErrorInternalServerError)?;

    let address_content = AddressContent {
        address,
        extrinsics: extrinsics
            .into_iter()
            .map(move |e| {
                let seconds = block_timestamps
                    .get(&BlockId::Hash(e.block.clone()))
                    .and_then(|system_time| {
                        Some(
                            system_time
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .ok()?
                                .as_secs(),
                        )
                    });
                ExtrinsicContent {
                    timestamp: seconds,
                    extr: e.without_json,
                }
            })
            .collect(),
    };

    let page = templates
        .get("entities/address.html")
        .ok_or_else(|| ErrorInternalServerError("Could not find template"))?
        .render(&address_content);
    Ok(html_page(templates, page)?)
}
