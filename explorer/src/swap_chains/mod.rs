use actix_web::{error::*, *};
use block_pool::Pool;
use chrono::{DateTime, Utc};
use ramhorns::Ramhorns;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use crate::retry_blocking;

mod chains;
pub use chains::{Receiver, ReceiverRef, Sender, SenderRef};

mod drive;
mod evm;
mod storage;

pub type Balance = u128;

pub fn resources() -> impl Iterator<Item = Resource> {
    vec![
        web::resource("/swap_chains").to(index),
        web::resource("/swap_chains/chain_options.json").to(chain_options),
        web::resource("/swap_chains/validate_address.json").to(validate_address),
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
    let receiver = chains::receiver(&options.system_receive).map_err(ErrorBadRequest)?;

    let sender = chains::sender(&options.system_send).map_err(ErrorBadRequest)?;
    if !sender.is_valid(&options.send_address) {
        return Err(ErrorBadRequest(anyhow::anyhow!(
            "Invalid send address {}",
            &options.send_address
        )));
    }

    let (state, secret_sidecar) = receiver.create_receive_request(&id);
    let swap = Swap {
        id: id.clone(),
        state,
        user: options.0,
        public_sidecar: Default::default(),
        secret_sidecar: secret_sidecar.unwrap_or_default(),
        events: Default::default(),
    };

    storage::insert_swap(swap, pg)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(web::Json(id))
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Swap<S = Sidecar> {
    id: String,
    state: SwapState,
    user: UserOptions,

    #[serde(default)]
    public_sidecar: Sidecar,
    #[serde(default)]
    secret_sidecar: S,

    events: VecDeque<TimedEvent>,
}

impl Swap {
    pub fn push_event(&mut self, event: Event) {
        self.events.push_front(TimedEvent {
            at: Utc::now(),
            event,
        });
    }

    pub fn transition_to(&mut self, state: SwapState) {
        let prev_state = core::mem::replace(&mut self.state, state);
        self.push_event(Event::TransitionedFromState(prev_state));
    }
}

impl<S> Swap<S> {
    pub fn strip_secrets(self) -> Swap<()> {
        Swap {
            secret_sidecar: (),

            id: self.id,
            state: self.state,
            user: self.user,
            public_sidecar: self.public_sidecar,
            events: self.events,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct TimedEvent {
    at: DateTime<Utc>,
    event: Event,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Sidecar(HashMap<String, serde_json::Value>);

impl Sidecar {
    pub fn with_mut<R, T: DeserializeOwned + Serialize + Default>(
        &mut self,
        key: &str,
        f: impl FnOnce(&mut T) -> R,
    ) -> anyhow::Result<R> {
        let mut t = self
            .0
            .remove(key)
            .map(|json| serde_json::from_value(json))
            .transpose()?
            .unwrap_or_default();

        let r = f(&mut t);

        self.0.insert(key.to_string(), serde_json::to_value(t)?);
        Ok(r)
    }

    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> anyhow::Result<()> {
        self.0.insert(key.to_string(), serde_json::to_value(value)?);
        Ok(())
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> anyhow::Result<Option<T>> {
        match self.0.get(key) {
            None => Ok(None),
            Some(v) => Ok(Some(serde_json::from_value(v.clone())?)),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SwapState {
    #[serde(rename_all = "camelCase")]
    AwaitingReceive(PaymentRequest),

    #[serde(rename_all = "camelCase")]
    Finalizing {},

    #[serde(rename_all = "camelCase")]
    Sending {
        // This is kept as a string because serde_json does not support u128.
        // https://github.com/serde-rs/json/issues/625
        amount_str: String,
    },

    #[serde(rename_all = "camelCase")]
    Finished { txn_id: String, txn_link: String },
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PaymentRequest {
    #[serde(rename_all = "camelCase")]
    Metamask {
        chain_id: u32,
        erc20_decimals: u8,
        transactions: Vec<evm::Transaction>,
    },

    #[serde(rename_all = "camelCase")]
    Simple {
        payment_request: String,
        receive_address: String,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Event {
    TransitionedFromState(SwapState),
    Generic(String, serde_json::Value),
}

impl Event {
    fn generic(name: &str, ser: impl Serialize) -> anyhow::Result<Event> {
        Ok(Event::Generic(name.to_string(), serde_json::to_value(ser)?))
    }
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
) -> actix_web::Result<web::Json<Swap<()>>> {
    let swap = {
        if let Some(found) = find_and_drive(id.clone(), pg)
            .await
            .map_err(ErrorInternalServerError)?
        {
            found
        } else {
            return Err(ErrorNotFound(anyhow::anyhow!("No swap with id {}", id)));
        }
    };

    Ok(web::Json(swap.strip_secrets()))
}

async fn find_and_drive(
    id: String,
    pg: web::Data<Pool<postgres::Client>>,
) -> anyhow::Result<Option<Swap>> {
    retry_blocking(move || {
        storage::run_locked(&mut pg.take(), &id, |swap| {
            drive::drive(
                swap,
                chains::receiver(&swap.user.system_receive)?,
                chains::sender(&swap.user.system_send)?,
            )
        })
    })
    .await
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ValidateRequest {
    chain: String,
    address: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ValidateResponse {
    valid: bool,
}

async fn validate_address(
    request: web::Json<ValidateRequest>,
) -> actix_web::Result<web::Json<ValidateResponse>> {
    let sender = chains::sender(&request.chain).map_err(ErrorBadRequest)?;
    let valid = sender.is_valid(&request.address);

    Ok(web::Json(ValidateResponse { valid }))
}
