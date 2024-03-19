use sqlx::{mysql::MySqlConnectOptions, Executor, MySql, MySqlPool, Pool};
use std::process::{Command, Output};

async fn connect_to_database() -> Result<Pool<MySql>, sqlx::Error> {
    // Configure database connection options
    let opts: MySqlConnectOptions = "mysql://root:@localhost:3306/music".parse()?;

    // Attempt to connect to the database
    match MySqlPool::connect_with(opts.clone()).await {
        Ok(connection) => Ok(connection),
        Err(err) => {
            // If connection fails, attempt to start the database server
            eprintln!(
                "Failed to connect to the database, attempting to reconnect: {}",
                err
            );
            start_database_server().await?;

            // Retry connecting
            MySqlPool::connect_with(opts).await
        }
    }
}

async fn start_database_server() -> Result<(), std::io::Error> {
    let os = std::env::consts::OS;
    let mut output = Command::new("").output();

    match os {
        "linux" => output = start_db_unix(),
        "windows" => output = start_db_windows(),
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Unsupported operating system: {os}",
            ));
        }
    };

    let out = output?;

    // Check if command execution was successful
    if out.status.success() {
        println!("Database server started successfully");
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to start database server: {:?}", out.stderr),
        ))
    }
}

fn start_db_unix() -> Result<Output, std::io::Error> {
    // Command to start MariaDB server, adjust according to your setup
    Command::new("sudo")
        .arg("service")
        .arg("mysql")
        .arg("start")
        .output()
}

fn start_db_windows() -> Result<Output, std::io::Error> {
    // Command to start MySQL service on Windows
    Command::new("net").arg("start").arg("mariadb").output()
}

struct Artist {
    name: String,
}
impl Artist {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

struct Album {
    title: String,
    artist: Artist,
    release_year: u32,
    media_type: String,
}

struct Song {
    title: String,
    artist: Artist,
    release_year: u32,
    media_type: String,
}
impl Song {
    pub fn new(title: String, artist: Artist, release_year: u32, media_type: String) -> Self {
        Self { title, artist, release_year, media_type }
    }
}

#[tokio::main]
async fn main() {
    match connect_to_database().await {
        Ok(pool) => {
            println!("Connected to the database");

        },
        Err(err) => eprintln!("Failed to connect to the database: {}", err),
    }
}

async fn query_db(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM Songs").fetch_all(pool).await?;
    for row in rows {
        let artist = row.get("Artist");
        let song = Song::new(row.get("Title"), artist, row.get("ReleaseYear"), row.get("MediaType"));
        println!("{:?}", song);
    }
    Ok(())
}