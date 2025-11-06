use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "grow")]
#[command(about = "A markdown viewer server", long_about = None)]
pub struct Args {
    /// Port number (default: auto-find from 8080)
    #[arg(long, default_value_t = 8080)]
    pub port: u16,

    /// Host to listen on
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    /// Directory to serve
    #[arg(default_value = ".")]
    pub directory: PathBuf,
}
