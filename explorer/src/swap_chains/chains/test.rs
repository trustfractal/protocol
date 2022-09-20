use ::serde::{Deserialize, Serialize};
use chrono::*;

use super::*;
use crate::swap_chains::*;

pub struct Test;

impl Chain for Test {
    fn info(&self) -> ChainInfo {
        ChainInfo {
            id: String::from("test"),
            name: String::from("Test"),
        }
    }
}

impl Receiver for Test {
    fn create_receive_request(&self, _id: &str) -> (SwapState, Option<Sidecar>) {
        (
            SwapState::AwaitingReceive(PaymentRequest::Simple {
                payment_request: "test:abcdef".to_string(),
                receive_address: "abcdef".to_string(),
            }),
            None,
        )
    }

    fn has_received(&self, swap: &mut Swap) -> anyhow::Result<bool> {
        swap.public_sidecar
            .with_mut("test_receive", |s: &mut TestReceiveSidecar| {
                let now = Utc::now();
                match s.will_receive_at {
                    None => {
                        s.will_receive_at = Some(now + Duration::seconds(10));
                        false
                    }
                    Some(at) => at <= now,
                }
            })
    }

    fn finalized_amount(&self, swap: &mut Swap) -> anyhow::Result<Option<Balance>> {
        swap.public_sidecar
            .with_mut("test_receive", |s: &mut TestReceiveSidecar| {
                let now = Utc::now();
                match s.will_finalize_at {
                    None => {
                        s.will_finalize_at = Some(now + Duration::seconds(10));
                        None
                    }
                    Some(at) if at <= now => Some(1234),
                    Some(_) => None,
                }
            })
    }

    fn after_finalized(&self, _swap: &mut Swap, _amount: Balance) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Default)]
struct TestReceiveSidecar {
    will_receive_at: Option<DateTime<Utc>>,
    will_finalize_at: Option<DateTime<Utc>>,
}

impl Sender for Test {
    fn send(&self, swap: &mut Swap, received_amount: Balance) -> anyhow::Result<SwapState> {
        let send_to = &swap.user.send_address;
        Ok(SwapState::Finished {
            txn_id: format!("send_to:{}/{}", send_to, received_amount),
            txn_link: format!("https://example.com/{}/{}", send_to, received_amount),
        })
    }
}

#[derive(Deserialize, Serialize, Default)]
struct TestSendSidecar {}
