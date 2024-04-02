use std::io;

use sqlx::MySqlPool;

use crate::database::connect_to_database;
use app::App;
use error::ArgumentError;

mod app;
mod database;
mod error;
mod popup;
mod song;
mod text_box;
mod tui;

pub fn parse_args(args: Vec<String>) -> Result<(), ArgumentError> {
    if args.len() == 2 {
        match args[1].as_str() {
            "-h" | "--help" => {
                print_help();
                Ok(())
            }
            "-v" | "--version" => {
                print_version();
                Ok(())
            }
            _ => Err(ArgumentError::InvalidArgument),
        }
    } else {
        Err(ArgumentError::InvalidNumberOfArguments)
    }
}

pub async fn initialize() -> Result<(), sqlx::Error> {
    const URL: &str = "mysql://root:@localhost:3306/music";
    let pool = connect_to_database(URL).await?;
    run_tui(pool).await?;
    Ok(())
}

fn print_help() {
    println!(
        "\nUsage: music-library [OPTIONS]\n
    \nOptions: 
    \n  <NONE> \t\tRun music library
    \n  -v, --version \tPrint version information
    \n  -h, --help \t\tPrint help (you are here)\n"
    );
}

fn print_version() {
    println!(
        "\nmusic-library v{} \n
    Written by Jack Lee\n
    Source: github.com/kcajeel/music-library\n",
        env!("CARGO_PKG_VERSION")
    );
}

async fn run_tui(pool: MySqlPool) -> io::Result<()> {
    let mut terminal = tui::init()?;
    let app_result = App::new(pool).run(&mut terminal).await;
    tui::restore()?;
    app_result
}
