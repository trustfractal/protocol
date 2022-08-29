use actix_web::{error::*, *};
use block_pool::Pool;
use chrono::{DateTime, Utc};
use ramhorns::Ramhorns;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

mod chains;
pub use chains::{Receiver, ReceiverRef, Sender, SenderRef};

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

    let (state, secret_sidecar) = receiver.create_receive_request();
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Event {
    TransitionedFromState(SwapState),
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
    if let Some(test) = test_data::get(&id) {
        return Ok(web::Json(test.strip_secrets()));
    }

    if let Some(found) = find_and_drive(id.clone(), pg)
        .await
        .map_err(ErrorInternalServerError)?
    {
        return Ok(web::Json(found.strip_secrets()));
    }

    Err(ErrorNotFound(anyhow::anyhow!("No swap with id {}", id)))
}

async fn find_and_drive(
    id: String,
    pg: web::Data<Pool<postgres::Client>>,
) -> anyhow::Result<Option<Swap>> {
    let found = storage::find_by_id(id.clone(), pg.clone()).await?;

    if let Some(mut result) = found {
        let driven = drive::drive(
            result.clone(),
            chains::receiver(&result.user.system_receive)?,
            chains::sender(&result.user.system_send)?,
        )
        .await?;
        if driven != result {
            result = storage::update(driven, pg).await?;
        }

        return Ok(Some(result));
    }

    Ok(None)
}