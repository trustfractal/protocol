use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

#[rpc]
pub trait HealthApi {
    #[rpc(name = "ready")]
    fn ready(&self) -> Result<u64>;
}

/// A struct that implements the `SillyRpc`
pub struct Health;

impl HealthApi for Health {
    fn ready(&self) -> Result<u64> {
        Err(jsonrpc_core::Error {
            code: jsonrpc_core::ErrorCode::InternalError,
            data: None,
            message: "Node is synchronizing".to_string(),
        })
    }
}
