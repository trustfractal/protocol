use actix_web::{error::*, *};
use ramhorns::Ramhorns;
use serde::{Deserialize, Serialize};

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

async fn create_swap(options: web::Json<CreateSwap>) -> actix_web::Result<impl Responder> {
    log::info!("{:?}", &options);

    Ok(web::Json(String::from("test-started")))
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Swap {
    id: String,
    state: SwapState,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
enum SwapState {
    #[serde(rename_all = "camelCase")]
    AwaitingReceive {
        payment_request: String,
        receive_address: String,
    },
}

async fn swap_page(templates: web::Data<Ramhorns>) -> actix_web::Result<HttpResponse> {
    let page = templates
        .get("swap_chains/swap.html")
        .ok_or_else(|| ErrorInternalServerError("Could not find template"))?
        .source();
    crate::pages::html_page(templates.clone(), page)
}

async fn get_swap(web::Path((id,)): web::Path<(String,)>) -> actix_web::Result<impl Responder> {
    Ok(web::Json(Swap {
        id,
        state: SwapState::AwaitingReceive {
            payment_request: String::from("bitcoincash:qpq0v9prnnvlf9ewflx0tekdlltwahv6asgvpact83"),
            receive_address: String::from("qpq0v9prnnvlf9ewflx0tekdlltwahv6asgvpact83"),
        },
    }))
}
