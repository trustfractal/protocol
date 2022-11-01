use super::*;

pub fn drive(swap: &mut Swap, receiver: ReceiverRef, sender: SenderRef, pg: &mut postgres::Client) -> anyhow::Result<()> {
    match &swap.state {
        SwapState::AwaitingReceive { .. } => drive_receive(swap, receiver),
        SwapState::Finalizing { .. } => drive_finalizing(swap, receiver),
        SwapState::Sending { amount_str } => {
            let amount = amount_str.parse()?;
            drive_sending(swap, receiver, sender, amount, pg)
        },
        SwapState::AttemptingSend { .. } => Ok(()),
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
    pg: &mut postgres::Client,
) -> anyhow::Result<()> {
    swap.transition_to(SwapState::AttemptingSend {});

    let modified_row_count = insert_swap_send_attempt(&swap.id, pg)?;

    if modified_row_count == 1 {
        receiver.after_finalized(swap, amount)?;
        let new_state = sender.send(swap, amount)?;
        swap.transition_to(new_state);
    }

    Ok(())
}

fn insert_swap_send_attempt(swap_id: &String, pg: &mut postgres::Client) -> anyhow::Result<u64> {
    let do_insert = |pg: &mut postgres::Client| {
        pg.execute("INSERT INTO swap_send_attempts (swap_id) VALUES ($1) ON CONFLICT DO NOTHING", &[swap_id])
    };

    let modified_row_count = do_insert(pg);

    if let Err(e) = modified_row_count {
        if let Some(db_error) = e.as_db_error() {
            if db_error.message() == "relation \"swap_send_attempts\" does not exist" {
                pg.execute(
                    "CREATE TABLE swap_send_attempts (
                        swap_id TEXT PRIMARY KEY NOT NULL
                    )",
                    &[],
                )?;
                return Ok(do_insert(pg)?);
            }
        }

        anyhow::bail!(e);
    }

    Ok(modified_row_count.unwrap())
}
