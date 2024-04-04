// This is the program's model of a Song. Most of this is self explanatory.

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Song {
    pub id: u32,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub release_year: i32,
    pub media_type: String,
}
impl Song {
    pub fn new(
        id: u32,
        title: &str,
        artist: &str,
        album: &str,
        release_year: i32,
        media_type: &str,
    ) -> Self {
        Self {
            id: id,
            title: title.to_owned(),
            artist: artist.to_owned(),
            album: album.to_owned(),
            release_year: release_year,
            media_type: media_type.to_owned(),
        }
    }

    pub fn default() -> Self {
        Self {
            id: 0,
            title: "Never Gonna Give You Up".to_owned(),
            artist: "Rick Astley".to_owned(),
            album: "Whenever You Need Somebody".to_owned(),
            release_year: 1987,
            media_type: "Vinyl".to_owned(),
        }
    }
}
