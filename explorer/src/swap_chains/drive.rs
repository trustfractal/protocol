use actix_web::web;

use super::*;

pub async fn drive(swap: Swap, receiver: ReceiverRef, sender: SenderRef) -> anyhow::Result<Swap> {
    match &swap.state {
        SwapState::AwaitingReceive { .. } => drive_receive(swap, receiver).await,
        SwapState::Finalizing { .. } => drive_finalizing(swap, receiver).await,
        SwapState::Sending { amount_str } => {
            let amount = amount_str.parse()?;
            drive_sending(swap, receiver, sender, amount).await
        }
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

async fn drive_finalizing(mut swap: Swap, receiver: ReceiverRef) -> anyhow::Result<Swap> {
    web::block(move || {
        if let Some(amount) = receiver.finalized_amount(&mut swap)? {
            swap.transition_to(SwapState::Sending {
                amount_str: amount.to_string(),
            });
        }

        anyhow::Ok(swap)
    })
    .await
    .map_err(anyhow::Error::new)
}

async fn drive_sending(
    mut swap: Swap,
    receiver: ReceiverRef,
    sender: SenderRef,
    amount: Balance,
) -> anyhow::Result<Swap> {
    receiver.after_finalized(&mut swap, amount)?;
    let new_state = sender.send(&mut swap, amount)?;
    swap.transition_to(new_state);
    Ok(swap)
}
