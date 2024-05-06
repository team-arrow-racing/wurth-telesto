mod adapter;

use std::ptr::addr_of_mut;

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
    /// Baud rate.
    #[arg(default_value_t = 115200)]
    baud: u32,
}

#[derive(Subcommand)]
enum Commands {
    /// Send data to configured address.
    ///
    /// You may send data using escaped strings such as \uXXXX and \xNN.
    Send { data: String },
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

    let mut queue_response = Queue::new();
    let mut queue_event = Queue::new();

    let (mut radio, mut ingress) = Radio::new(
        tx,
        rx,
        unsafe { addr_of_mut!(queue_response).as_mut().unwrap() },
        unsafe { addr_of_mut!(queue_event).as_mut().unwrap() },
    );

    tokio::task::spawn(async move {
        ingress.ingest().await.unwrap();
    });

    match args.subcommand {
        Commands::Send { data } => {
            let output = unescape::unescape(&data).unwrap();
            radio.send(output.as_bytes()).await.unwrap()
        }
        Commands::Reset => radio.reset().await.unwrap(),
        _ => todo!(),
    }

    println!("Finished...");
}
