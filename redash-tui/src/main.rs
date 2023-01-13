use std::error::Error;

use clap::Parser;
use pancurses::{endwin, initscr, noecho};
use redash_client::client::Client;
use tui::run;

pub mod app;
pub mod tui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    #[arg(long, value_name = "HOST")]
    host: Option<String>,

    #[arg(long, value_name = "PORT")]
    port: Option<u16>,
}

static DEFAULT_PORT: u16 = 6379;
static DEFAULT_HOST: &str = "127.0.0.1";
fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let port = &cli.port.unwrap_or_else(|| DEFAULT_PORT);
    let host = &cli.host.unwrap_or_else(|| String::from(DEFAULT_HOST));

    let mut client = Client::new(host, *port);
    client.connect().unwrap();

    let window = initscr();
    window.refresh();
    window.keypad(true);

    noecho();
    run(&client, &window)?;

    window.clear();
    endwin();
    Ok(())
}
