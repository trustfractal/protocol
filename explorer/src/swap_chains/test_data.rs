use super::{Swap, SwapState};

pub fn get(id: &str) -> Option<Swap> {
    match id {
        "test-started" => Some(Swap {
            id: id.to_string(),
            state: SwapState::AwaitingReceive {
                payment_request: String::from(
                    "bitcoincash:qpq0v9prnnvlf9ewflx0tekdlltwahv6asgvpact83",
                ),
                receive_address: String::from("qpq0v9prnnvlf9ewflx0tekdlltwahv6asgvpact83"),
            },
        }),
        "test-finalizing" => Some(Swap {
            id: id.to_string(),
            state: SwapState::Finalizing {},
        }),
        _ => None,
    }
}
