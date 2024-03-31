use std::{env, error::Error};

use music_library::{initialize, parse_args};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        parse_args(args)?;
    } else {
        initialize().await?;
    }
    Ok(())
}
