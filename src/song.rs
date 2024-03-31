#[derive(Debug, sqlx::FromRow)]
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
}
