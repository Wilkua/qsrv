use clap::{ArgAction, Parser};
use eyre::Result;
use hyper::server::conn::http1;
use qsrv::responders::FileResolver;
use std::net::SocketAddr;
use time::macros::format_description;
use tokio::net::TcpListener;
use tracing::{error, info, Level};
use tracing_subscriber::fmt::{
    time::UtcTime,
    Subscriber,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CommandLine {
    /// Document root where served files are located
    #[arg(short, long)]
    document_root: Option<String>,

    // Port to listen on
    #[arg(short, long)]
    port: Option<u16>,

    /// Reduce the amount of logging to only errors
    #[arg(short, long, action=ArgAction::SetTrue)]
    quiet: Option<bool>,

    /// Suppress all log messages
    #[arg(short, long, action=ArgAction::SetTrue, default_value="false")]
    silent: bool,

    // Enable verbose logging
    #[arg(short, long, action=ArgAction::Count, default_value="0")]
    verbose: u8,
}

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
