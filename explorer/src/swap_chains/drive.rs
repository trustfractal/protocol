use actix_web::{error::BlockingError, web};

use super::{Event, Receiver, Swap, SwapState};

pub async fn drive(swap: Swap, receiver: Box<dyn Receiver>) -> anyhow::Result<Swap> {
    match &swap.state {
        SwapState::AwaitingReceive { .. } => drive_receive(swap, receiver).await,
        unhandled => {
            unimplemented!("unhandled: {:?}", unhandled);
        }
    }
}

async fn drive_receive(mut swap: Swap, receiver: Box<dyn Receiver>) -> anyhow::Result<Swap> {
    web::block(move || {
        if receiver.has_received(&mut swap)? {
            let prev_state = core::mem::replace(&mut swap.state, SwapState::Finalizing {});
            swap.push_event(Event::TransitionedFromState(prev_state));
        }
        Ok(swap)
    })
    .await
    .map_err(|e: BlockingError<anyhow::Error>| anyhow::anyhow!(e))
}
