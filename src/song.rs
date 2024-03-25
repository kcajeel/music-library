#[derive(Debug, sqlx::FromRow)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub release_year: i32,
    pub media_type: String,
}
impl Song {
    #[allow(dead_code)]
    pub fn new(
        title: String,
        artist: String,
        album: String,
        release_year: i32,
        media_type: String,
    ) -> Self {
        Self {
            title,
            artist,
            album,
            release_year,
            media_type,
        }
    }
}