use actix_web::{error::*, *};
use block_pool::Pool;
use ramhorns::Ramhorns;
use serde::{Deserialize, Serialize};

use crate::retry_blocking;

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
struct ChainInfo {
    id: String,
    name: String,
}

async fn chain_options() -> actix_web::Result<impl Responder> {
    let test_chain = ChainInfo {
        id: String::from("test"),
        name: String::from("Test"),
    };

    Ok(web::Json(ChainOptions {
        system_receive: vec![test_chain.clone()],
        system_send: vec![test_chain],
    }))
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct CreateSwap {
    system_receive: String,
    system_send: String,
    send_address: String,
}

async fn create_swap(
    options: web::Json<CreateSwap>,
    pg: web::Data<Pool<postgres::Client>>,
) -> actix_web::Result<impl Responder> {
    let id = bs58::encode(rand::random::<u64>().to_string()).into_string();
    let receiver = receiver(&options.system_receive).map_err(ErrorInternalServerError)?;

    let state = receiver.create_receive_request();
    let swap = Swap {
        id: id.clone(),
        state,
    };

    insert_swap(swap, pg).await?;

    Ok(web::Json(id))
}

fn receiver(id: &str) -> anyhow::Result<Box<dyn Receiver>> {
    match id {
        "test" => Ok(Box::new(Test)),
        _ => anyhow::bail!("Unrecognized receiver {}", id),
    }
}

trait Receiver {
    fn create_receive_request(&self) -> SwapState;
}

struct Test;

impl Receiver for Test {
    fn create_receive_request(&self) -> SwapState {
        SwapState::AwaitingReceive {
            payment_request: "test:abcdef".to_string(),
            receive_address: "abcdef".to_string(),
        }
    }
}

async fn insert_swap(swap: Swap, pg: web::Data<Pool<postgres::Client>>) -> actix_web::Result<()> {
    retry_blocking(move || {
        let pg = &mut pg.take();

        let id = &swap.id;
        let json = serde_json::to_value(&swap)?;

        let do_insert = |pg: &mut postgres::Client| {
            pg.execute("INSERT INTO swaps (id, json) VALUES ($1, $2)", &[id, &json])
        };

        if let Err(e) = do_insert(pg) {
            if let Some(db_error) = e.as_db_error() {
                if db_error.message() == "relation \"swaps\" does not exist" {
                    pg.execute(
                        "CREATE TABLE swaps (
                            id TEXT PRIMARY KEY NOT NULL,
                            json JSON NOT NULL
                        )",
                        &[],
                    )?;
                    do_insert(pg)?;
                    return Ok(());
                }
            }

            anyhow::bail!(e);
        }

        Ok(())
    })
    .await
    .map_err(|e: anyhow::Error| ErrorInternalServerError(e))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Swap {
    id: String,
    state: SwapState,
}

#[derive(Deserialize, Serialize, Debug)]
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
) -> actix_web::Result<impl Responder> {
    if let Some(test) = test_data::get(&id) {
        return Ok(HttpResponse::Ok().json(test));
    }

    let queried = retry_blocking(move || {
        let pg = &mut pg.take();
        let queried = pg.query_opt("SELECT json FROM swaps WHERE id = $1", &[&id])?;
        let row = match queried {
            Some(r) => r,
            None => return Ok(None),
        };
        let json = row.get("json");
        let swap: Swap = serde_json::from_value(json)?;

        Ok(Some(swap))
    })
    .await
    .map_err(|e: anyhow::Error| ErrorInternalServerError(e))?;
    if let Some(queried) = queried {
        return Ok(HttpResponse::Ok().json(queried));
    }

    Ok(HttpResponse::NotFound().finish())
}
