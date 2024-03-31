mod database;
mod song;

use crate::database::{connect_to_database, get_all_songs};

#[tokio::main]
#[allow(unused_variables)]
async fn main() -> Result<(), sqlx::Error> {
    const URL: &str = "mysql://root:@localhost:3306/music";
    let pool = connect_to_database(URL).await?;
    let songs = get_all_songs(&pool).await?;
    println!("{:?}", songs);
    Ok(())
}
