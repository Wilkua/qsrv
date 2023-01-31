use crate::TcpStream;
use eyre::Result;
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::net::SocketAddr;
use tracing::{error, info};

pub struct HttpServer {
    addr: SocketAddr,
}

const SERVER_TOKEN: Token = Token(0);

impl HttpServer {
    pub fn new(addr: impl Into<SocketAddr>) -> Self {
        HttpServer {
            addr: addr.into(),
        }
    }

    pub fn run<F>(self, mut f: F) -> Result<()>
        where F: FnMut(TcpStream),
    {
        let mut poll = Poll::new()?;
        let mut listener = TcpListener::bind(self.addr)?;
        poll.registry()
            .register(&mut listener, SERVER_TOKEN, Interest::READABLE)?;

        info!("Server listening on {}", self.addr);

        let mut events = Events::with_capacity(128);
        loop {
            poll.poll(&mut events, None)?;
            for event in events.iter() {
                match event.token() {
                    SERVER_TOKEN => match listener.accept() {
                        Ok((stream, _)) => f(stream),
                        Err(e) => error!("Failed to accept connection: {}", e),
                    },
                    token => info!("Other token - {:?}", token),
                }
            }
        }
    }
}
