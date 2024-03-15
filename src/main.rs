use clap::{ArgAction, Parser};
use eyre::Result;
use qsrv::{
    handle_request,
    work_queue,
     HttpServer,
};
use std::{
    sync::Arc,
    thread,
};
use time::macros::format_description;
use tracing::{error, info, Level};
use tracing_subscriber::fmt::{
    Subscriber,
    time::UtcTime,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of concurrent threads for handling connections
    #[arg(short, long)]
    concurrency: Option<usize>,

    /// Document root where served files are located
    #[arg(short, long)]
    document_root: Option<String>,

    /// Reduce the amount of logging to only errors
    #[arg(short, long, action=ArgAction::SetTrue)]
    quiet: Option<bool>,

    /// Suppress all log messages
    #[arg(short, long, action=ArgAction::SetTrue, default_value="false")]
    silent: bool,
}

#[derive(Clone)]
enum Work<T> {
    Job(T),
    #[allow(dead_code)]
    Quit
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.silent {
        let log_level = match args.quiet {
            Some(true) => Level::ERROR,
            Some(false) | None => Level::INFO,
        };

        let t = UtcTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"));
        let subscriber = Subscriber::builder()
            .with_timer(t)
            .with_max_level(log_level)
            .finish();

        tracing::subscriber::set_global_default(subscriber)?;
    }

    let path = Arc::new(args.document_root.unwrap_or(".".into()).clone());
    info!("document root set to \"{}\"", path);

    let avail_threads = usize::from(thread::available_parallelism()?);
    let threads = args.concurrency.unwrap_or(avail_threads);
    let threads = if threads == 0 {
        avail_threads
    } else {
        threads
    };

    info!("using {} threads", threads);

    let (mut snd, recv) = work_queue::make_queue(threads * 4);

    let mut workers = Vec::with_capacity(threads);
    for _ in 0..threads {
        let recv = recv.clone();
        let path = Arc::clone(&path);
        workers.push(thread::spawn(move || {
            for work in recv {
                match work {
                    Work::Job(s) => {
                        match handle_request(s, &path) {
                            Ok(_) => (),
                            Err(e) => error!("Request error: {:?}", e),
                        }
                    },
                    Work::Quit => break,
                }
            }
        }));
    }

    let server = HttpServer::new(([0, 0, 0, 0], 3000));
    server.run(|stream| {
        snd.dispatch(Work::Job(Arc::new(stream)));
    })?;

    Ok(())
}

