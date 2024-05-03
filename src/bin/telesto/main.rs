mod adapter;

use clap::{Parser, Subcommand};
use heapless::spsc::Queue;
use tokio_serial::SerialPortBuilderExt;
use wurth_telesto::Radio;

#[derive(Parser)]
pub struct Cli {
    #[clap(subcommand)]
    subcommand: Commands,
    #[arg(value_name = "SERIAL PORT")]
    port: String,
    /// Baud rate. Defaults to 115200
    #[arg(default_value_t = 115200)]
    baud: u32,
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

    let stream = tokio_serial::new(args.port, args.baud)
        .open_native_async()
        .unwrap();

    let (tx, rx) = adapter::make_split_stream(stream);

    let mut queue_response = Queue::new();
    let mut queue_event = Queue::new();

    let radio = Radio::new(tx, rx, &mut queue_response, &mut queue_event);

    match args.subcommand {
        _ => todo!(),
    }
}
