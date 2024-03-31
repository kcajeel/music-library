mod database;
mod song;

use crate::database::{connect_to_database, get_all_songs};
use music_library::parse_args;
use std::{env, error::Error};

#[tokio::main]
#[allow(unused_variables)]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        parse_args(args)?;
    } else {
        const URL: &str = "mysql://root:@localhost:3306/music";
        let pool = connect_to_database(URL).await?;
        let songs = get_all_songs(&pool).await?;
        println!("{:?}", songs);
    }
    Ok(())
}
