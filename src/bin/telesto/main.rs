use clap::{Parser, Subcommand};
use tokio_serial::SerialPortBuilderExt;

#[derive(Parser)]
pub struct Cli {
    #[clap(subcommand)]
    subcommand: Commands,
    #[arg(value_name = "SERIAL PORT")]
    port: String,
    /// Baud rate. Defaults to 115200
    #[arg(default_value_t = 115200)]
    baud: u64,
}

#[derive(Subcommand)]
enum Commands {
    /// Reset module.
    Reset,
    /// Shutdown module.
    Shutdown,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let mut port = tokio_serial::new(args.port, 115200).open_native_async().unwrap();
}
