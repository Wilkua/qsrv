use eyre::Result;
use hyper::server::conn::http1;
use qsrv::{
    responders::FileResolver,
    CommandLine, Parser,
};
use std::net::SocketAddr;
use time::macros::format_description;
use tokio::net::TcpListener;
use tracing::{error, info, Level};
use tracing_subscriber::fmt::{
    time::UtcTime,
    Subscriber,
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = CommandLine::parse();

    if !args.silent {
        let log_level = match args.verbose {
            0 => Level::INFO,
            1 => Level::DEBUG,
            _ => Level::TRACE,
        };

        let log_level = match args.quiet {
            Some(true) => Level::ERROR,
            Some(false) | None => log_level,
        };

        let t = UtcTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"));
        let subscriber = Subscriber::builder()
            .with_ansi(false)
            .with_timer(t)
            .with_max_level(log_level)
            .finish();

        tracing::subscriber::set_global_default(subscriber)?;
    }

    // let path = Arc::new(args.document_root.unwrap_or(".".into()).clone());
    let path = args.document_root.unwrap_or(".".into());
    info!("document root set to \"{}\"", path);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port.unwrap_or(3000)));
    let listener = TcpListener::bind(addr).await?;
    info!("server listening on {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;

        let root_path = path.clone();
        tokio::task::spawn(async move {
            let svc = FileResolver::new(&root_path).unwrap();
            if let Err(e) = http1::Builder::new()
                .serve_connection(stream, svc).await
            {
                error!("Error serving connection: {:?}", e);
            }
        });
    }

    // Ok(())
}
