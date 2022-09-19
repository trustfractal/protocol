use super::*;

use web3::{types::*, *};

pub struct EvmMintable {
    info: ChainInfo,
    web3: Web3<transports::Http>,
    url: String,
}

impl EvmMintable {
    pub fn new(url: String, info: ChainInfo) -> Self {
        EvmMintable {
            info,
            web3: Web3::new(transports::Http::new(&url).unwrap()),
            url,
        }
    }
}

impl Chain for EvmMintable {
    fn info(&self) -> ChainInfo {
        self.info.clone()
    }
}

impl Sender for EvmMintable {
    fn send(&self, _swap: &mut Swap, _amount: Balance) -> anyhow::Result<SwapState> {
        block_on(async {
            dbg!(
                self.web3
                    .eth()
                    .block(BlockId::Number(BlockNumber::Latest))
                    .await?
            );

            unimplemented!("send");
        })
    }
}

fn block_on<F: core::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}
