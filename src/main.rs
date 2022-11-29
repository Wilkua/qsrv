use eyre::Result;
use qsrv::{
    responders::FileServer,
    HttpRequest, HttpServer, Responder
};
use std::{
    io::Write,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
    thread,
};
use time::macros::format_description;

use tracing::{error, info, Level};
use tracing_subscriber::fmt::{
    Subscriber,
    time::UtcTime,
};
use work_pool::WorkPool;

fn handle_request(mut stream: Arc<TcpStream>) -> Result<()> {
    let Some(mut stream) = Arc::get_mut(&mut stream) else {
        return Ok(());
    };

    let req = HttpRequest::new(&mut stream);

    let file_server = FileServer::new(".")?;
    let response = file_server.handle_request(&req)?;

    stream.write(&response.as_bytes())?;

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
    let t = UtcTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"));
    let subscriber = Subscriber::builder()
        .with_timer(t)
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr)?;

    info!("Server listening on http://localhost:3000/");

    let server = HttpServer::new(([0, 0, 0, 0], 3000));
    server.run()?;

    let threads = usize::from(thread::available_parallelism()?);
    let queue_cap = threads * 4;
    info!("using {} threads with a queue capacity of {}", threads, queue_cap);
    let mut pool = WorkPool::new(threads, Some(queue_cap)).expect("Failed to generate work pool");

    pool.set_executor_and_start(|work| {
        match handle_request(work) {
            Ok(_) => (),
            Err(e) => error!("Error while processing request:{}", e),
        };
    });

    for stream in listener.incoming() {
        match stream {
            Ok(s) => pool.dispatch(Arc::new(s)),
            Err(e) => error!("error: {:?}", e),
        };
    }

    Ok(())
}
