use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "mvu")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_args() {
        let args = Args::try_parse_from(&["mvu"]).unwrap();
        assert_eq!(args.port, 8080);
        assert_eq!(args.host, "0.0.0.0");
        assert_eq!(args.directory, PathBuf::from("."));
    }

    #[test]
    fn test_custom_port() {
        let args = Args::try_parse_from(&["mvu", "--port", "3000"]).unwrap();
        assert_eq!(args.port, 3000);
    }

    #[test]
    fn test_custom_host() {
        let args = Args::try_parse_from(&["mvu", "--host", "127.0.0.1"]).unwrap();
        assert_eq!(args.host, "127.0.0.1");
    }

    #[test]
    fn test_custom_directory() {
        let args = Args::try_parse_from(&["mvu", "/tmp"]).unwrap();
        assert_eq!(args.directory, PathBuf::from("/tmp"));
    }

    #[test]
    fn test_all_custom_args() {
        let args = Args::try_parse_from(&["mvu", "--port", "9090", "--host", "localhost", "/var/www"]).unwrap();
        assert_eq!(args.port, 9090);
        assert_eq!(args.host, "localhost");
        assert_eq!(args.directory, PathBuf::from("/var/www"));
    }
}
