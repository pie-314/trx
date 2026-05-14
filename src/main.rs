mod config;
mod fuzzy;
mod managers;
mod ui;
mod updater;

use color_eyre::Result;
use managers::Package;
use ratatui::crossterm::{
    cursor::{Hide, Show},
    event::DisableMouseCapture,
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{init, restore};
use std::io::{self};
use std::sync::mpsc;
use ui::app::App;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("trx version {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" | "-h" => {
                println!("trx - A Modern Cross-Platform Package Manager TUI");
                println!("\nUsage: trx [OPTIONS]");
                println!("\nOptions:");
                println!("  -v, --version    Print version information");
                println!("  -h, --help       Print help information");
                println!("\nKeybindings (inside TUI):");
                println!("  q                Quit");
                println!("  Tab              Switch between Search, Installed, and Updates tabs");
                println!("  Shift+Tab        Switch backwards between tabs");
                println!("  e                Edit search query (Search tab)");
                println!("  Space            Select/unselect packages");
                println!("  i                Install selected packages");
                println!("  x                Remove selected packages");
                println!("  U                Full system upgrade");
                println!("  R                Refresh package databases");
                println!("  ?                Toggle help overlay");
                return Ok(());
            }
            _ => {}
        }
    }

    color_eyre::install()?;
    let mut terminal = init();
    execute!(std::io::stdout(), ratatui::crossterm::event::EnableMouseCapture)?;
    let (result_tx, result_rx) = mpsc::channel::<(String, Vec<Package>)>();
    let app_result = App::new(result_tx.clone(), result_rx).run(&mut terminal);
    execute!(std::io::stdout(), DisableMouseCapture)?;
    restore();

    match app_result {
        Ok(Some(url)) => {
            println!("Downloading update...");
            match updater::update_self(&url) {
                Ok(_) => {
                    println!("Update complete. Please restart trx.");
                }
                Err(e) => {
                    eprintln!("Failed to update: {}", e);
                }
            }
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn execute_external_command(
    terminal: &mut ratatui::DefaultTerminal,
    cmd: &str,
    args: &[&str],
) -> Result<()> {
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        Show,
        DisableMouseCapture
    )?;

    println!("\n{}", "=".repeat(40));
    println!(" RUNNING: {} {}", cmd, args.join(" "));
    println!("{}\n", "=".repeat(40));

    let status = std::process::Command::new(cmd).args(args).status();

    println!("\n{}", "=".repeat(40));
    match status {
        Ok(s) if s.success() => println!(" STATUS: Success"),
        Ok(s) => println!(" STATUS: Failed ({:?})", s),
        Err(e) => println!(" ERROR: {}", e),
    }
    println!("{}", "=".repeat(40));
    println!("\nPress Enter to return to trx...");

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    execute!(terminal.backend_mut(), Hide)?;
    terminal.clear()?;

    Ok(())
}
