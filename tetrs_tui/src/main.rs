mod game_input_handlers;
mod game_mods;
mod game_renderers;
mod terminal_app;

use std::io::{self, Write};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// A custom starting layout for Combo mode, encoded in binary, by 4-wide rows.
    /// Example: "▀▄▄▀" => 0b_1001_0110 = 150
    ///          => `./tetrs_tui -c 150`.
    #[arg(short, long)]
    combo_layout: Option<u16>,
    /// A custom starting board for **Custom** mode, encoded in binary, by 10-wide rows.
    /// Example: "█▀ ▄██▀ ▀█" => 0b_1100111011_1001110001 = 982815
    ///          => `./tetrs_tui --custom_start=982815`.
    #[arg(long)]
    custom_start: Option<u128>,
    /// Whether to enable the combo bot in combo mode.
    #[arg(short, long)]
    enable_combo_bot: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let stdout = io::BufWriter::new(io::stdout());
    let mut app = terminal_app::TerminalApp::new(
        stdout,
        args.combo_layout,
        args.custom_start,
        args.enable_combo_bot,
    );
    std::panic::set_hook(Box::new(|panic_info| {
        if let Ok(mut file) = std::fs::File::create("tetrs_tui_error_message.txt") {
            let _ = file.write(panic_info.to_string().as_bytes());
            // let _ = file.write(std::backtrace::Backtrace::force_capture().to_string().as_bytes());
        }
    }));
    let msg = app.run()?;
    println!("{msg}");
    Ok(())
}
