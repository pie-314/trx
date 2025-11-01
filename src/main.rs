mod pacman;
mod ui;

use color_eyre::Result;
use pacman::Package;
use ratatui::init;
use std::sync::mpsc;
use ui::app::App;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = init();
    let (result_tx, result_rx): (mpsc::Sender<Vec<Package>>, mpsc::Receiver<Vec<Package>>) =
        mpsc::channel();
    let app_result = App::new(result_tx.clone(), result_rx).run(terminal);
    ratatui::restore();
    app_result
}
/*fn main() {
    println!("1 for searching");
    println!("2 for current packages");
    println!("3 for exit");
    loop {
        println!("choice: ");
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");

        let trimmed_input = user_input.trim();

        if trimmed_input == "1" {
            pacman::list_packages_installed();
        } else if trimmed_input == "2" {
            println!("package name: ");
            let mut package = String::new();
            io::stdin()
                .read_line(&mut package)
                .expect("Failed to read line");

            let trimmed_package = package.trim();
            pacman::search(trimmed_package);
        } else {
            println!("bye bye !!!");
            break;
        }
    }
}*/
