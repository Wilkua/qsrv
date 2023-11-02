pub use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CommandLine {
    /// Document root where served files are located
    #[arg(short, long)]
    pub document_root: Option<String>,

    // Port to listen on
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Reduce the amount of logging to only errors
    #[arg(short, long, action=ArgAction::SetTrue)]
    pub quiet: Option<bool>,

    /// Suppress all log messages
    #[arg(short, long, action=ArgAction::SetTrue, default_value="false")]
    pub silent: bool,

    // Enable verbose logging
    #[arg(short, long, action=ArgAction::Count, default_value="0")]
    pub verbose: u8,
}
