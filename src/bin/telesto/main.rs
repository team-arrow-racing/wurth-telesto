use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[clap(subcommand)]
    subcommand: Commands,
    #[arg(value_name = "SERIAL PORT")]
    port: String,
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
}
