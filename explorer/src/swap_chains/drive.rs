use super::*;

pub fn drive(swap: &mut Swap, receiver: ReceiverRef, sender: SenderRef) -> anyhow::Result<()> {
    match &swap.state {
        SwapState::AwaitingReceive { .. } => drive_receive(swap, receiver),
        SwapState::Finalizing { .. } => drive_finalizing(swap, receiver),
        SwapState::Sending { amount_str } => {
            let amount = amount_str.parse()?;
            drive_sending(swap, receiver, sender, amount)
        }
        SwapState::Finished { .. } => Ok(()),
    }
}

fn drive_receive(swap: &mut Swap, receiver: ReceiverRef) -> anyhow::Result<()> {
    if receiver.has_received(swap)? {
        swap.transition_to(SwapState::Finalizing {});
    }
    Ok(())
}

fn drive_finalizing(swap: &mut Swap, receiver: ReceiverRef) -> anyhow::Result<()> {
    if let Some(amount) = receiver.finalized_amount(swap)? {
        swap.transition_to(SwapState::Sending {
            amount_str: amount.to_string(),
        });
    }

    Ok(())
}

fn drive_sending(
    swap: &mut Swap,
    receiver: ReceiverRef,
    sender: SenderRef,
    amount: Balance,
) -> anyhow::Result<()> {
    receiver.after_finalized(swap, amount)?;
    let new_state = sender.send(swap, amount)?;
    swap.transition_to(new_state);
    Ok(())
}
