mod pacman;
mod ui;

use color_eyre::Result;
use pacman::Package;
use ratatui::crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{init, restore};
use std::io::{self};
use std::sync::mpsc;
use ui::app::App;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = init();
    let (result_tx, result_rx): (mpsc::Sender<Vec<Package>>, mpsc::Receiver<Vec<Package>>) =
        mpsc::channel();
    let app_result = App::new(result_tx.clone(), result_rx).run(&mut terminal);
    restore();
    app_result
}

pub fn execute_external_command(
    terminal: &mut ratatui::DefaultTerminal,
    cmd: &str,
    args: &[&str],
) -> Result<()> {
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    execute!(terminal.backend_mut(), Show)?;

    println!("\nExecuting: {} {}\n", cmd, args.join(" "));
    let status = std::process::Command::new(cmd).args(args).status();
    println!(
        "\nCommand finished: {:?}\nPress Enter to continue...",
        status
    );

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    execute!(terminal.backend_mut(), Hide)?;
    terminal.clear()?;

    Ok(())
}
