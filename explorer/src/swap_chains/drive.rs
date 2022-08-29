use actix_web::web;

use super::{Receiver, Sender, Swap, SwapState};

pub async fn drive(
    swap: Swap,
    receiver: Box<dyn Receiver>,
    sender: Box<dyn Sender>,
) -> anyhow::Result<Swap> {
    match &swap.state {
        SwapState::AwaitingReceive { .. } => drive_receive(swap, receiver).await,
        SwapState::Finalizing { .. } => drive_finalizing(swap, receiver, sender).await,
        SwapState::Finished { .. } => Ok(swap),
    }
}

async fn drive_receive(mut swap: Swap, receiver: Box<dyn Receiver>) -> anyhow::Result<Swap> {
    web::block(move || {
        if receiver.has_received(&mut swap)? {
            swap.transition_to(SwapState::Finalizing {});
        }
        anyhow::Ok(swap)
    })
    .await
    .map_err(anyhow::Error::new)
}

async fn drive_finalizing(
    mut swap: Swap,
    receiver: Box<dyn Receiver>,
    sender: Box<dyn Sender>,
) -> anyhow::Result<Swap> {
    web::block(move || {
        if receiver.has_finalized(&mut swap)? {
            let finished = sender.send(&mut swap)?;
            swap.transition_to(finished);
        }
        anyhow::Ok(swap)
    })
    .await
    .map_err(anyhow::Error::new)
}
