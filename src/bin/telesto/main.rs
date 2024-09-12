use clap::{Parser, Subcommand};
use heapless::spsc::Queue;
use std::ptr::addr_of_mut;
use tokio_serial::SerialPortBuilderExt;
use wurth_telesto::{Event, Mode, Radio};

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
    /// Echo incomming data sent to the configured address.
    Echo,
    /// Enter standby.
    Standby,
    /// Receive signal strength of last received packet.
    Rssi,
    /// Transmit power.
    TxPower { power: u8 },
    /// Set channel.
    Channel { channel: u8 },
    /// Destination network ID.
    DestNet { id: u8 },
    /// Destination address.
    DestAddr { address: u8 },
    /// Operating mode.
    Mode { mode: Mode },
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

    let (tx, rx) = wurth_telesto::tokio::split_stream(stream);

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
        Commands::Echo => loop {
            let event = radio.poll_event().await;
            if let Event::DataReceived = event.command() {
                let data = &event.data()[0..event.data().len() - 1];
                let strength = *event.data().last().unwrap() as i8;

                println!("Got data: {:#02x?} with RSSI: {}dBm", data, strength);
                radio.send(event.data()).await.unwrap();
                println!("Sent response.");
            }
        },
        Commands::Standby => radio.standby().await.unwrap(),
        Commands::Rssi => {
            let rssi = radio.rssi().await.unwrap();
            if rssi == 0x80 {
                println!("No RSSI available");
            } else {
                let rssi = rssi as i8;
                println!("RSSI: {}dBm", rssi);
            }
        }
        Commands::TxPower { power } => radio.tx_power(power).await.unwrap(),
        Commands::Channel { channel } => radio.channel(channel).await.unwrap(),
        Commands::DestNet { id } => radio.destination_net(id).await.unwrap(),
        Commands::DestAddr { address } => radio.destination_address(address).await.unwrap(),
        Commands::Mode { mode } => radio.mode(mode).await.unwrap(),
        _ => todo!(),
    }

    println!("Finished...");
}
