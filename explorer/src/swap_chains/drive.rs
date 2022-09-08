use actix_web::web;

use super::*;

pub async fn drive(swap: Swap, receiver: ReceiverRef, sender: SenderRef) -> anyhow::Result<Swap> {
    match &swap.state {
        SwapState::AwaitingReceive { .. } => drive_receive(swap, receiver).await,
        SwapState::Finalizing { .. } => drive_finalizing(swap, receiver, sender).await,
        SwapState::Finished { .. } => Ok(swap),
    }
}

async fn drive_receive(mut swap: Swap, receiver: ReceiverRef) -> anyhow::Result<Swap> {
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
    receiver: ReceiverRef,
    sender: SenderRef,
) -> anyhow::Result<Swap> {
    web::block(move || {
        if let Some(_) = &swap.after_txns_submitted {
            for txn in &swap.receiver_txns {
                receiver.ensure_submitted(txn)?;
            }
            for txn in &swap.sender_txns {
                sender.ensure_submitted(txn)?;
            }

            let next_state = swap.after_txns_submitted.take().unwrap();
            swap.transition_to(next_state);
            return Ok(swap);
        }

        if let Some(balance) = receiver.finalized_amount(&mut swap)? {
            let txns = receiver.post_finalize_txns(&mut swap)?;
            swap.receiver_txns = txns;

            let (after_submitted, txns) = sender.send_txns(&mut swap, balance)?;
            swap.after_txns_submitted = Some(after_submitted);
            swap.sender_txns = txns;

            return Ok(swap);
        }

        anyhow::Ok(swap)
    })
    .await
    .map_err(anyhow::Error::new)
}
