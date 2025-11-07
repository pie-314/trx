mod pacman;
mod ui;

use color_eyre::Result;
use ratatui::crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::process::Command;

use pacman::Package;
use ratatui::Terminal;
use ratatui::init;
use ratatui::prelude::CrosstermBackend;
use std::io;
use std::sync::mpsc;
use ui::app::App;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = init(); // Make terminal mutable
    let (result_tx, result_rx): (mpsc::Sender<Vec<Package>>, mpsc::Receiver<Vec<Package>>) =
        mpsc::channel();
    let app_result = App::new(result_tx.clone(), result_rx).run(terminal);
    ratatui::restore();
    app_result
}

// Put this in src/ui/app.rs instead, or make it public
pub fn execute_external_command(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    cmd: &str,
    args: &[&str],
) -> Result<()> {
    // Suspend TUI
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    execute!(terminal.backend_mut(), Show)?;

    // Execute command
    println!("\nExecuting: {} {}\n", cmd, args.join(" "));
    let status = Command::new(cmd).args(args).status();
    println!(
        "\nCommand finished: {:?}\nPress Enter to continue...",
        status
    );

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // Resume TUI
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    execute!(terminal.backend_mut(), Hide)?;
    terminal.clear()?;

    Ok(())
}
