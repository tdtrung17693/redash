use std::{env, error::Error};

use clap::Parser;
use constants::FOCUS_COLOR;
use pancurses::{endwin, init_pair, initscr, noecho, raw, start_color, COLOR_CYAN};
use redash_client::client::Client;
use tui::run;

pub mod app;
pub mod constants;
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
    let current_esc_delay = match env::var("ESCDELAY") {
        Ok(v) => v,
        Err(_) => String::from("0"),
    };
    env::set_var("ESCDELAY", "0");
    let window = initscr();
    window.refresh();
    window.keypad(true);
    start_color();
    init_pair(FOCUS_COLOR as i16, COLOR_CYAN, 0);
    // window.nodelay(true);

    noecho();
    raw();
    run(&client, &window)?;

    window.clear();
    endwin();
    env::set_var("ESCDELAY", current_esc_delay);
    Ok(())
}
