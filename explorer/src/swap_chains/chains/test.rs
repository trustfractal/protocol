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
    fn create_receive_request(&self) -> SwapState {
        SwapState::AwaitingReceive {
            payment_request: "test:abcdef".to_string(),
            receive_address: "abcdef".to_string(),
        }
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
}

#[derive(Deserialize, Serialize, Default)]
struct TestReceiveSidecar {
    will_receive_at: Option<DateTime<Utc>>,
}

impl Sender for Test {}
