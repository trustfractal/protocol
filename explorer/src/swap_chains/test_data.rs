use super::{Swap, SwapState};

pub fn get(id: &str) -> Option<Swap> {
    let id = id.to_string();
    match id.as_ref() {
        "test-started" => Some(Swap {
            id,
            state: SwapState::AwaitingReceive {
                payment_request: String::from(
                    "bitcoincash:qpq0v9prnnvlf9ewflx0tekdlltwahv6asgvpact83",
                ),
                receive_address: String::from("qpq0v9prnnvlf9ewflx0tekdlltwahv6asgvpact83"),
            },
        }),
        "test-finalizing" => Some(Swap {
            id,
            state: SwapState::Finalizing {},
        }),
        "test-finished" => Some(Swap {
            id,
            state: SwapState::Finished {
                txn_id: String::from("6957432b57933c0a3e6f46661b79754affe3c1931fbf1d896443a3ef3a75dad1"),
                txn_link: String::from("https://blockchair.com/bitcoin-cash/transaction/6957432b57933c0a3e6f46661b79754affe3c1931fbf1d896443a3ef3a75dad1"),
            }
        }),
        _ => None,
    }
}
