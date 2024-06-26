// Database module contains all database functions for connecting and interacting with the DB

use crate::song::Song;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlQueryResult},
    MySql, MySqlPool, Pool,
};
use std::process::{Command, Output};

pub async fn connect_to_database(url: &str) -> Result<Pool<MySql>, sqlx::Error> {
    // Configure database connection options
    let opts: MySqlConnectOptions = match url.parse() {
        Ok(valid_url) => valid_url,
        Err(error) => panic!("Invalid URL. Error: {}", error),
    };
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
    // attempt to run the appropriate command depending on OS
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
    Command::new("mariadbd").output()
}

fn start_db_windows() -> Result<Output, std::io::Error> {
    // Command to start MySQL service on Windows
    Command::new("net").arg("start").arg("mariadb").output()
}

// Add a song to DB
pub async fn add_song(pool: &MySqlPool, new_song: Song) -> Result<MySqlQueryResult, sqlx::Error> {
    let result = sqlx::query("INSERT INTO Songs (id, title, artist, album, release_year, media_type) VALUES (?, ?, ?, ?, ?, ?)")
    .bind(0)
    .bind(new_song.title)
    .bind(new_song.artist)
    .bind(new_song.album)
    .bind(new_song.release_year)
    .bind(new_song.media_type)
    .execute(pool).await?;
    Ok(result)
}

// Search function to look for songs like a keyword
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

// gets every song
pub async fn get_all_songs(pool: &MySqlPool) -> Result<Vec<Song>, sqlx::Error> {
    let songs = sqlx::query_as!(Song, "SELECT * FROM Songs")
        .fetch_all(pool)
        .await?;
    Ok(songs)
}

// update a song with new info
pub async fn update_song(
    pool: &MySqlPool,
    song_id: u32,
    song_fields: Song,
) -> Result<MySqlQueryResult, sqlx::Error> {
    let result = sqlx::query("UPDATE Songs SET title = ?, artist = ?, album = ?, release_year = ?, media_type = ? WHERE id = ?").bind(song_fields.title).bind(song_fields.artist).bind(song_fields.album).bind(song_fields.release_year).bind(song_fields.media_type).bind(song_id).execute(pool).await?;
    Ok(result)
}

// delete a song by id
pub async fn delete_song(pool: &MySqlPool, song_id: u32) -> Result<MySqlQueryResult, sqlx::Error> {
    let result = sqlx::query("DELETE FROM Songs WHERE id = ?")
        .bind(song_id)
        .execute(pool)
        .await?;
    Ok(result)
}

// Tests for DB functions
#[cfg(test)]
mod tests {

    // TO RUN TESTS: YOU MUST SET DATABASE_URL TO music-test
    // cargo sqlx prepare -D mysql://root:@localhost:3306/music-test
    // Don't forget to prepare sqlx for the actual database when done running tests ;)

    // These tests are meant to be run in sequence; Don't just `cargo test`, run each one individually.

    use super::*;
    use crate::song::Song;

    const URL: &str = "mysql://root:@localhost:3306/music-test";

    // test song creation
    #[tokio::test]
    async fn test_create() {
        let pool = connect_to_database(URL).await.unwrap();
        let test_song = Song::new(0, "Testing", "Unit Tests", "Under Test", 2024, "N/A");
        let result = add_song(&pool, test_song).await.unwrap();
        println!("rows affected by create: {}", result.rows_affected());
        // assert the query executed
        assert!(result.rows_affected() == 1);
    }

    // test song retrieval
    #[tokio::test]
    async fn test_retrieve() {
        let pool = connect_to_database(URL).await.unwrap();
        let test_song = "Test";
        let matching_songs = get_songs_matching(&pool, test_song.to_owned())
            .await
            .unwrap();

        println!("Songs that match {}: {:?}", test_song, matching_songs);
        // assert the result contains keyword ignoring case
        assert!(
            matching_songs.get(0).unwrap().title.contains("Test")
                || matching_songs.get(0).unwrap().title.contains("test")
        );
    }

    // test updating a song
    #[tokio::test]
    async fn test_update() {
        let pool = connect_to_database(URL).await.unwrap();
        let updated_song = Song::new(0, "testing again", "Unit Tests", "Testing 2", 2024, "N/A");
        let song_list = get_songs_matching(&pool, "Testing".to_owned())
            .await
            .unwrap();
        let test_song = song_list.get(0).unwrap();
        println!("Song with id {} will be updated", test_song.id);

        let result = update_song(&pool, test_song.id, updated_song)
            .await
            .unwrap();
        println!("Rows affected by update: {:?}", result);
        // assert the update was sucessful
        assert!(
            result.rows_affected() == 1,
            "rows affected: {}",
            result.rows_affected()
        );
    }

    // test deleting a song
    #[tokio::test]
    async fn test_delete() {
        let pool = connect_to_database(URL).await.unwrap();
        let all_songs = get_all_songs(&pool).await.unwrap();

        println!("Songs before delete: {:?}", all_songs);
        let lowest_id = get_lowest_id(&all_songs);
        delete_song(&pool, lowest_id).await.unwrap();

        let new_songs = get_all_songs(&pool).await.unwrap();
        println!("Songs after delete: {:?}", new_songs);
        assert_ne!(new_songs.len(), all_songs.len()); // assert the total # of songs decreased
    }

    // helper function to get the song with lowest id
    fn get_lowest_id(songs: &Vec<Song>) -> u32 {
        let mut lowest = 999;
        for song in songs {
            if song.id < lowest {
                lowest = song.id
            }
        }
        lowest
    }
}
