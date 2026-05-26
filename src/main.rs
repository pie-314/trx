mod config;
mod fuzzy;
mod managers;
mod ui;
mod updater;

use color_eyre::Result;
use managers::Package;
use ratatui::crossterm::{
    cursor::{Hide, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
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
                let config = config::Config::load();
                let keys = &config.keys;
                println!("trx - A Modern Cross-Platform Package Manager TUI");
                println!("\nUsage: trx [OPTIONS]");
                println!("\nOptions:");
                println!("  -v, --version    Print version information");
                println!("  -h, --help       Print help information");
                println!("\nKeybindings (current configuration):");
                println!("  {:<16} Quit", keys.quit);
                println!("  {:<16} Switch to next tab", format!("{}/Tab", keys.tab_next));
                println!("  {:<16} Switch to previous tab", format!("{}/Shift+Tab", keys.tab_prev));
                println!("  {:<16} Edit search query (Search tab)", keys.search_edit);
                println!("  {:<16} Toggle package selection or settings", if keys.toggle_select == " " { "Space".to_string() } else { keys.toggle_select.clone() });
                println!("  {:<16} Install selected packages", keys.install);
                println!("  {:<16} Remove selected packages", keys.remove);
                println!("  {:<16} Update selected packages", keys.update);
                println!("  {:<16} Full system upgrade", keys.system_upgrade);
                println!("  {:<16} Refresh package databases", keys.refresh_db);
                println!("  {:<16} Toggle help overlay", keys.help);
                println!("  {:<16} Check for trx binary updates", keys.check_update);
                println!("\nMouse Support:");
                println!("  Full navigation, tab switching, and scrolling are supported.");
                return Ok(());
            }
            _ => {}
        }
    }

    color_eyre::install()?;
    let mut terminal = init();
    execute!(std::io::stdout(), EnableMouseCapture)?;
    let (result_tx, result_rx): (mpsc::Sender<(String, Vec<Package>)>, mpsc::Receiver<(String, Vec<Package>)>) =
        mpsc::channel();
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
        Err(e) => Err(e.into()),
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

    execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
    terminal::enable_raw_mode()?;
    execute!(terminal.backend_mut(), Hide)?;
    terminal.clear()?;

    Ok(())
}
