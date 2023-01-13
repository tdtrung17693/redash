use std::error::Error;

use clap::{Parser, Subcommand};

use crate::client::Client;
mod client;
mod errors;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    #[arg(long, value_name = "HOST")]
    host: Option<String>,

    #[arg(long, value_name = "PORT")]
    port: Option<u16>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
}

static DEFAULT_PORT: u16 = 6379;
static DEFAULT_HOST: &str = "127.0.0.1";

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let port = &cli.port.unwrap_or_else(|| DEFAULT_PORT);
    let host = &cli.host.unwrap_or_else(|| String::from(DEFAULT_HOST));

    let mut client = Client::new(host, *port);
    client.connect().unwrap();
    let res = (client).send_command("PING")?;
    println!("{:?}", res);
    Ok(())
}
