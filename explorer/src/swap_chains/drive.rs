use super::{Receiver, Swap, SwapState};
use crate::retry_blocking;

pub async fn drive(swap: &mut Swap, receiver: Box<dyn Receiver>) -> anyhow::Result<()> {
    match &swap.state {
        SwapState::AwaitingReceive { .. } => drive_receive(swap, receiver).await,
        unhandled => {
            unimplemented!("unhandled: {:?}", unhandled);
        }
    }
}

async fn drive_receive(_swap: &mut Swap, _receiver: Box<dyn Receiver>) -> anyhow::Result<()> {
    retry_blocking(|| Ok(())).await
}
