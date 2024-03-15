use crate::{
    responders::FileServer,
    HttpRequest,
    Responder,
    TcpStream,
};
use eyre::Result;
use std::{
    io::Write,
    sync::Arc,
};
use tracing::info;

pub fn handle_request(mut stream: Arc<TcpStream>, path: &str) -> Result<()> {
    let Some(mut stream) = Arc::get_mut(&mut stream) else {
        return Ok(());
    };

    let req = HttpRequest::new(&mut stream);

    let file_server = FileServer::new(path)?;
    let response = file_server.handle_request(&req)?;

    stream.write_all(&response.as_bytes())?;

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

