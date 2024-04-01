use crate::song::Song;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlQueryResult},
    MySql, MySqlPool, Pool,
};
use std::process::{Command, Output};
// Module Database contains all database functions for connecting and interacting with the DB

pub async fn connect_to_database(url: &str) -> Result<Pool<MySql>, sqlx::Error> {
    // Configure database connection options
    let opts: MySqlConnectOptions = url.parse()?;

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

pub async fn add_song(new_song: Song, pool: &MySqlPool) -> Result<MySqlQueryResult, sqlx::Error> {
    let result = sqlx::query("INSERT INTO Songs (id, title, artist, album, release_year, media_type) VALUES (?, ?, ?, ?, ?, ?)")
    .bind(0)
    .bind(new_song.title)
    .bind(new_song.artist)
    .bind(new_song.album)
    .bind(new_song.release_year)
    .bind(new_song.media_type)
    .execute(pool).await?;
    println!("{:?}", result);
    Ok(result)
}

pub async fn get_songs_matching(
    pool: &MySqlPool,
    keyword: String,
) -> Result<Vec<Song>, sqlx::Error> {
    let keyword_like = format!("%{}%", keyword);

    let ids: Vec<Song> = sqlx::query_as!(Song,
        "SELECT * FROM Songs WHERE title LIKE ? OR artist LIKE ? OR album LIKE ? OR release_year LIKE ? OR media_type LIKE ?",
        keyword_like,
        keyword_like,
        keyword_like,
        keyword_like,
        keyword_like
    )
    .fetch_all(pool)
    .await?;
    Ok(ids)
}

pub async fn get_all_songs(pool: &MySqlPool) -> Result<Vec<Song>, sqlx::Error> {
    let songs = sqlx::query_as!(
        Song,
        "SELECT id, title, artist, album, release_year, media_type FROM Songs"
    )
    .fetch_all(pool)
    .await?;
    Ok(songs)
}

pub async fn update_song(
    pool: &MySqlPool,
    song_id: u32,
    song_fields: Song,
) -> Result<MySqlQueryResult, sqlx::Error> {
    let result = sqlx::query("UPDATE Songs SET title = ?, artist = ?, album = ?, release_year = ?, media_type = ? WHERE id = ?").bind(song_fields.title).bind(song_fields.artist).bind(song_fields.album).bind(song_fields.release_year).bind(song_fields.media_type).bind(song_id).execute(pool).await?;
    Ok(result)
}

pub async fn delete_song(pool: &MySqlPool, song_id: u32) -> Result<MySqlQueryResult, sqlx::Error> {
    let result = sqlx::query("DELETE FROM Songs WHERE id = ?")
        .bind(song_id)
        .execute(pool)
        .await?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::song::Song;

    const URL: &str = "mysql://root:@localhost:3306/music-test";

    #[tokio::test]
    async fn test_create() {
        let pool = connect_to_database(URL).await.unwrap();
        let test_song = Song::new(0, "Testing", "Unit Tests", "Under Test", 2024, "N/A");
        let result = add_song(test_song, &pool).await.unwrap();
        println!("rows affected by create: {}", result.rows_affected());
        assert!(result.rows_affected() == 1);
    }

    #[tokio::test]
    async fn test_retrieve() {
        let pool = connect_to_database(URL).await.unwrap();
        let test_song = "Test";
        let matching_ids = get_songs_matching(&pool, test_song.to_owned())
            .await
            .unwrap();
        println!("Song id's that match {}: {:?}", test_song, matching_ids);
        assert!(matching_ids.len() == 1);
        assert!(*matching_ids.get(0).unwrap() == 2);
    }

    //I updated get_songs_matching and it broke my tests. I'll fix it later..
    #[tokio::test]
    async fn test_update() {
        let pool = connect_to_database(URL).await.unwrap();
        let updated_song = Song::new(0, "testing again", "Unit Tests", "Testing 2", 2024, "N/A");

        let id_list = get_songs_matching(&pool, "Testing".to_owned())
            .await
            .unwrap();
        assert!(id_list.len() == 1);
        let test_song_id = *id_list.get(0).unwrap();
        println!("Song with id {test_song_id} will be updated");

        let result = update_song(&pool, test_song_id, updated_song)
            .await
            .unwrap();
        println!("Rows affected by update: {:?}", result);
        assert!(result.rows_affected() == 1);
    }

    fn get_lowest_id(songs: &Vec<Song>) -> u32 {
        let mut lowest = 999;
        for song in songs {
            if song.id < lowest {
                lowest = song.id
            }
        }
        lowest
    }

    #[tokio::test]
    async fn test_delete() {
        let pool = connect_to_database(URL).await.unwrap();
        let all_songs = get_all_songs(&pool).await.unwrap();
        println!("Songs before delete: {:?}", all_songs);
        let lowest_id = get_lowest_id(&all_songs);
        delete_song(&pool, lowest_id).await.unwrap();
        let new_songs = get_all_songs(&pool).await.unwrap();
        println!("Songs after delete: {:?}", new_songs);
        assert_ne!(new_songs.len(), all_songs.len());
    }
}
