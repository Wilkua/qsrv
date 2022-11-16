use std::net::SocketAddr;
use tracing::info;

use eyre::Result;

pub struct HttpServer {
    addr: SocketAddr,
}

impl HttpServer {
    pub fn new(addr: impl Into<SocketAddr>) -> Self {
        HttpServer {
            addr: addr.into(),
        }
    }

    pub fn run(self) -> Result<()> {
        info!("Server listening {:?}", self.addr);
        Ok(())
    }
}
