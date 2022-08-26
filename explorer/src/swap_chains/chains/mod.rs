use super::{ChainInfo, SwapState};

pub trait Chain {
    fn info(&self) -> ChainInfo;
}

pub trait Receiver: Chain {
    fn create_receive_request(&self) -> SwapState;
}

pub trait Sender: Chain {}

pub fn receivers() -> impl Iterator<Item = Box<dyn Receiver>> {
    vec![Box::new(Test) as Box<dyn Receiver>].into_iter()
}

pub fn senders() -> impl Iterator<Item = Box<dyn Sender>> {
    vec![Box::new(Test) as Box<dyn Sender>].into_iter()
}

pub fn receiver(id: &str) -> anyhow::Result<Box<dyn Receiver>> {
    receivers()
        .find(|r| r.info().id == id)
        .ok_or_else(|| anyhow::anyhow!("Unrecognized receiver {}", id))
}

struct Test;

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
}

impl Sender for Test {}
