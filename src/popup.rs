// this file contains the popup menu widget logic

use crate::{
    database::{add_song, update_song},
    song::Song,
    text_box::{InputMode, TextBox},
};
use sqlx::MySqlPool;

// popup modes related to App mode
#[derive(Debug, Clone)]
pub enum PopupMode {
    New,
    Edit,
}

// Popup struct stores all state info
#[derive(Debug, Clone)]
pub struct Popup {
    mode: PopupMode, // mode
    song_id: u32,    // id for song to edit or create
    // text boxes for each input field
    pub title_box: TextBox,
    pub artist_box: TextBox,
    pub album_box: TextBox,
    pub release_year_box: TextBox,
    pub media_type_box: TextBox,
}

impl Popup {
    pub fn new(mode: PopupMode, song_id: u32) -> Self {
        // self explanatory
        Self {
            mode,
            song_id,
            title_box: TextBox::new("Title".to_owned()),
            artist_box: TextBox::new("Artist".to_owned()),
            album_box: TextBox::new("Album".to_owned()),
            release_year_box: TextBox::new("Year".to_owned()),
            media_type_box: TextBox::new("Media Type".to_owned()),
        }
    }
    pub async fn submit(&mut self, pool: &MySqlPool) {
        // submit all boxes and store a song from the input
        self.submit_all_boxes();
        let new_song = self.get_song_from_input();
        match self.mode {
            // if mode is New, add new song and print any errors
            PopupMode::New => {
                match add_song(pool, new_song).await {
                    Ok(_) => (),
                    Err(error) => eprintln!("Error adding song: {error}"),
                };
            }
            // if Edit mode, update the song and print any errors
            PopupMode::Edit => {
                match update_song(pool, self.song_id, new_song).await {
                    Ok(_) => (),
                    Err(error) => eprintln!("Error updating song: {error}"),
                };
            }
        }
    }

    // submits all text boxes
    fn submit_all_boxes(&mut self) {
        self.title_box.submit_message();
        self.artist_box.submit_message();
        self.album_box.submit_message();
        self.release_year_box.submit_message();
        self.media_type_box.submit_message();
    }

    // clears all input fields in text boxes
    pub fn clear_all_boxes(&mut self) {
        self.title_box.clear_input();
        self.artist_box.clear_input();
        self.album_box.clear_input();
        self.release_year_box.clear_input();
        self.media_type_box.clear_input();
    }

    // returns a song from textbox input
    // this function is only called when all boxes have input
    fn get_song_from_input(&self) -> Song {
        Song::new(
            0,
            &self.title_box.get_mesages().pop().unwrap(),
            &self.artist_box.get_mesages().pop().unwrap(),
            &self.album_box.get_mesages().pop().unwrap(),
            self.release_year_box
                .get_mesages()
                .pop()
                .unwrap()
                .parse::<i32>()
                .unwrap(),
            &self.media_type_box.get_mesages().pop().unwrap(),
        )
    }

    // returns true if any text boxes are in editing mode
    pub fn are_any_boxes_editing_mode(&self) -> bool {
        let mut result = false;
        if self.title_box.get_input_mode() == InputMode::Editing {
            result = true;
        } else if self.artist_box.get_input_mode() == InputMode::Editing {
            result = true;
        } else if self.album_box.get_input_mode() == InputMode::Editing {
            result = true;
        } else if self.release_year_box.get_input_mode() == InputMode::Editing {
            result = true;
        } else if self.media_type_box.get_input_mode() == InputMode::Editing {
            result = true;
        }
        result
    }

    // returns true if all boxes have some text in them
    pub fn do_all_boxes_have_text(&self) -> bool {
        let mut result = false;
        if (self.title_box.get_input().len() > 0)
            && (self.artist_box.get_input().len() > 0)
            && (self.album_box.get_input().len() > 0)
            && (self.release_year_box.get_input().len() > 0)
            && (self.media_type_box.get_input().len() > 0)
        {
            result = true;
        }
        result
    }

    // sets all textbox input modes to the passed mode
    pub fn set_all_input_modes(&mut self, new_mode: InputMode) {
        self.title_box.set_input_mode(new_mode.clone());
        self.artist_box.set_input_mode(new_mode.clone());
        self.album_box.set_input_mode(new_mode.clone());
        self.release_year_box.set_input_mode(new_mode.clone());
        self.media_type_box.set_input_mode(new_mode);
    }

    // pushes the data from the song's fields to each text box
    pub fn populate_textboxes_with_song(&mut self, song: &Song) {
        self.title_box.set_input(song.title.clone());
        self.artist_box.set_input(song.artist.clone());
        self.album_box.set_input(song.album.clone());
        self.release_year_box
            .set_input(song.release_year.to_string());
        self.media_type_box.set_input(song.media_type.clone());
    }

    pub fn get_popup_mode(&self) -> PopupMode {
        self.mode.clone()
    }

    pub fn set_song_id(&mut self, new_id: u32) {
        self.song_id = new_id;
    }
}
