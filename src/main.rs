mod song;

use crate::song::Song;
use sqlx::{mysql::MySqlConnectOptions, MySql, MySqlPool, Pool};
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

#[allow(unused_assignments)]
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

#[allow(non_snake_case)]
async fn get_songs(pool: &MySqlPool) -> Result<Vec<Song>, sqlx::Error> {
    let songs = sqlx::query_as!(Song, r#"SELECT * FROM Songs"#)
        .fetch_all(pool)
        .await?;
    Ok(songs)
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    match connect_to_database().await {
        Ok(pool) => {
            println!("Connected to the database");
            let songs = get_songs(&pool).await?;
            println!("{:#?}", songs);
        }
        Err(err) => eprintln!("Failed to connect to the database: {}", err),
    }

    Ok(())
}
