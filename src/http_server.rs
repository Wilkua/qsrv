use crate::TcpStream;
use eyre::Result;
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::io::Write;
use std::{
    collections::HashMap,
    io::{ErrorKind, Read},
    net::SocketAddr,
};
use tracing::{debug, error, info, trace};

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

    pub fn run<F>(self, _f: F) -> Result<()>
        where F: Fn(TcpStream),
    {
        let mut poll = Poll::new()?;
        let mut listener = TcpListener::bind(self.addr)?;
        poll.registry()
            .register(&mut listener, SERVER_TOKEN, Interest::READABLE)?;

        info!("server listening on {}", self.addr);

        let mut events = Events::with_capacity(128);
        let mut socks = 0;
        let mut streams = HashMap::new();
        loop {
            poll.poll(&mut events, None)?;
            for event in events.iter() {
                match event.token() {
                    SERVER_TOKEN => match listener.accept() {
                        Ok((mut stream, peer)) => {
                            debug!("accepted connection from {}", peer);
                            let token = Token(socks + 1);
                            poll.registry().register(&mut stream, token, Interest::READABLE)?;
                            streams.insert(token, stream);
                            socks += 1;
                        },
                        Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                        Err(e) => error!("failed to accept connection: {}", e),
                    },
                    token if event.is_readable() => {
                        trace!("readable event for token {}", token.0);
                        let stream = streams.get_mut(&token).unwrap();
                        let mut buf = [0u8; 256];
                        match stream.read(&mut buf) {
                            Ok(bytes_read) => {
                                debug!("bytes_read = {}", bytes_read);
                                if bytes_read == 0 {
                                    continue;
                                }
                                let d = String::from_utf8(buf.to_vec()).unwrap_or("?".to_string());
                                info!("read ({}) {}", d.len(), d);
                                // info!("Read {}", String::from_utf8(Vec::from(&buf)).unwrap());
                                let _ = stream.write("HTTP/1.1 200 Ok\r\nConnection: close\r\n\r\n".as_bytes());

                                let mut stream = streams.remove(&token).unwrap();
                                let _ = poll.registry().deregister(&mut stream);
                                drop(stream);
                            },
                            Err(e) => {
                                error!("Bad things happened: {}", e);
                            },
                        }
                        // match streams.remove(&token) {
                        //     Some(stream) => f(stream),
                        //     None => error!("received event for non-existent socket ({})", token.0),
                        // }
                    }
                    token => {
                        info!("token = {:?}, event = {:?}", token, event);
                    },
                }
            }
        }
    }
}
