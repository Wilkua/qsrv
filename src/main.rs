use eyre::Result;
use qsrv::{file_server::FileServer, HttpRequest, Responder};
use std::io::Write;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use tracing::{error, info, Level, span};
use tracing_subscriber::fmt::Subscriber;

fn handle_request(stream: TcpStream) -> Result<()> {
    let mut req = HttpRequest::new(stream);

    let file_server = FileServer::new(".")?;
    let response = file_server.handle_request(&req)?;

    req.stream.write(&response.as_bytes())?;

    info!("{} {} \"{}\" {} bytes",
        &response.status,
        &response.method,
        &req.path,
        match &response.body {
            Some(b) => b.len(),
            None => 0,
        });

    Ok(())
}

fn main() -> Result<()> {
    let subscriber = Subscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr)?;

    info!("Server listening on http://localhost:3000/");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                thread::spawn(|| {
                    match handle_request(s) {
                        Ok(_) => (),
                        Err(e) => error!("Error while processing request:{}", e),
                    }
                });
            },
            Err(e) => {
                error!("error: {:?}", e);
            }
        };
    }

    Ok(())
}
