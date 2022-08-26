use actix_web::{error::*, *};
use block_pool::Pool;
use ramhorns::Ramhorns;
use serde::{Deserialize, Serialize};

mod chains;
pub use chains::Receiver;

mod drive;
mod storage;
mod test_data;

pub fn resources() -> impl Iterator<Item = Resource> {
    vec![
        web::resource("/swap_chains").to(index),
        web::resource("/swap_chains/chain_options.json").to(chain_options),
        web::resource("/swap_chains/create.json").to(create_swap),
        web::resource("/swap_chains/{id}.json").to(get_swap),
        web::resource("/swap_chains/{id}").to(swap_page),
    ]
    .into_iter()
}

async fn index(templates: web::Data<Ramhorns>) -> actix_web::Result<HttpResponse> {
    let page = templates
        .get("swap_chains/index.html")
        .ok_or_else(|| ErrorInternalServerError("Could not find template"))?
        .source();
    crate::pages::html_page(templates.clone(), page)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ChainOptions {
    system_receive: Vec<ChainInfo>,
    system_send: Vec<ChainInfo>,
}

#[derive(Serialize, Clone)]
pub struct ChainInfo {
    id: String,
    name: String,
}

async fn chain_options() -> actix_web::Result<impl Responder> {
    let receivers = chains::receivers().map(|r| r.info());
    let senders = chains::senders().map(|r| r.info());

    Ok(web::Json(ChainOptions {
        system_receive: receivers.collect(),
        system_send: senders.collect(),
    }))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct UserOptions {
    system_receive: String,
    system_send: String,
    send_address: String,
}

async fn create_swap(
    options: web::Json<UserOptions>,
    pg: web::Data<Pool<postgres::Client>>,
) -> actix_web::Result<impl Responder> {
    let id = bs58::encode(rand::random::<u64>().to_string()).into_string();
    let receiver = chains::receiver(&options.system_receive).map_err(ErrorInternalServerError)?;

    let state = receiver.create_receive_request();
    let swap = Swap {
        id: id.clone(),
        state,
        user: options.0,
    };

    storage::insert_swap(swap, pg).await?;

    Ok(web::Json(id))
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Swap {
    id: String,
    state: SwapState,
    user: UserOptions,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SwapState {
    #[serde(rename_all = "camelCase")]
    AwaitingReceive {
        payment_request: String,
        receive_address: String,
    },

    #[serde(rename_all = "camelCase")]
    Finalizing {},

    #[serde(rename_all = "camelCase")]
    Finished { txn_id: String, txn_link: String },
}

async fn swap_page(templates: web::Data<Ramhorns>) -> actix_web::Result<HttpResponse> {
    let page = templates
        .get("swap_chains/swap.html")
        .ok_or_else(|| ErrorInternalServerError("Could not find template"))?
        .source();
    crate::pages::html_page(templates.clone(), page)
}

async fn get_swap(
    web::Path((id,)): web::Path<(String,)>,
    pg: web::Data<Pool<postgres::Client>>,
) -> actix_web::Result<web::Json<Swap>> {
    if let Some(test) = test_data::get(&id) {
        return Ok(web::Json(test));
    }

    if let Some(found) = find_and_drive(id.clone(), pg)
        .await
        .map_err(ErrorInternalServerError)?
    {
        return Ok(web::Json(found));
    }

    Err(ErrorNotFound(anyhow::anyhow!("No swap with id {}", id)))
}

async fn find_and_drive(
    id: String,
    pg: web::Data<Pool<postgres::Client>>,
) -> anyhow::Result<Option<Swap>> {
    let found = storage::find_by_id(id.clone(), pg.clone()).await?;

    if let Some(mut result) = found {
        let mut driven = result.clone();
        drive::drive(&mut driven, chains::receiver(&result.user.system_receive)?).await?;
        if driven != result {
            result = storage::update(driven, pg).await?;
        }

        return Ok(Some(result));
    }

    Ok(None)
}
