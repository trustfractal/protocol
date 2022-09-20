use super::*;

pub struct EvmBurner {
    info: ChainInfo,
    chain_id: u32,
}

impl EvmBurner {
    pub fn new(info: ChainInfo, chain_id: String) -> anyhow::Result<Self> {
        Ok(EvmBurner {
            info,
            chain_id: chain_id.parse()?,
        })
    }
}

impl Chain for EvmBurner {
    fn info(&self) -> ChainInfo {
        self.info.clone()
    }
}

impl Receiver for EvmBurner {
    fn create_receive_request(&self) -> (SwapState, Option<Sidecar>) {
        let state = SwapState::AwaitingReceive(PaymentRequest::Metamask {
            chain_id: self.chain_id,
        });
        (state, None)
    }

    fn has_received(&self, _swap: &mut Swap) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn finalized_amount(&self, _swap: &mut Swap) -> anyhow::Result<Option<Balance>> {
        unimplemented!("finalized_amount");
    }

    fn after_finalized(&self, _: &mut Swap, _: Balance) -> anyhow::Result<()> {
        unimplemented!("after_finalized");
    }
}
