mod adapter;

use std::ptr::addr_of_mut;

use clap::{Parser, Subcommand};
use heapless::spsc::Queue;
use tokio_serial::SerialPortBuilderExt;
use wurth_telesto::{Event, Frame, Radio, Response};

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
        .data_bits(tokio_serial::DataBits::Eight)
        .stop_bits(tokio_serial::StopBits::One)
        .parity(tokio_serial::Parity::None)
        .open_native_async()
        .unwrap();

    let (tx, rx) = adapter::make_split_stream(stream);

    static mut QUEUE_RESPONSE: Queue<Frame<Response>, 2> = Queue::new();
    static mut QUEUE_EVENT: Queue<Frame<Event>, 16> = Queue::new();

    let (mut radio, mut ingress) = Radio::new(
        tx,
        rx,
        unsafe { addr_of_mut!(QUEUE_RESPONSE).as_mut().unwrap() },
        unsafe { addr_of_mut!(QUEUE_EVENT).as_mut().unwrap() },
    );

    tokio::task::spawn(async move {
        ingress.ingest().await.unwrap();
    });

    match args.subcommand {
        Commands::Reset => radio.reset().await.unwrap(),
        _ => todo!(),
    }

    println!("Finished...");
}
