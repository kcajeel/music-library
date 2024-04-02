// popup menu widget for editing and creating songs

use crate::{
    database::{add_song, update_song},
    song::Song,
    text_box::{InputMode, TextBox},
};
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};
use sqlx::MySqlPool;

#[derive(Debug, Clone)]
pub enum PopupMode {
    Create,
    Update,
}

#[derive(Debug, Clone)]
pub struct Popup {
    pub mode: PopupMode,
    pub song_id: u32,
    pub title_box: TextBox,
    pub artist_box: TextBox,
    pub album_box: TextBox,
    pub release_year_box: TextBox,
    pub media_type_box: TextBox,
}

impl Popup {
    pub fn new(mode: PopupMode, song_id: u32) -> Self {
        Self {
            mode,
            song_id,
            title_box: TextBox::new(),
            artist_box: TextBox::new(),
            album_box: TextBox::new(),
            release_year_box: TextBox::new(),
            media_type_box: TextBox::new(),
        }
    }
    pub async fn submit(&mut self, pool: &MySqlPool) {
        let new_song = Song::new(
            0,
            self.title_box.get_input(),
            &self.artist_box.get_input(),
            &self.album_box.get_input(),
            self.release_year_box.get_input().parse::<i32>().unwrap(),
            &self.media_type_box.get_input(),
        );
        match self.mode {
            PopupMode::Create => {
                add_song(pool, new_song).await.unwrap();
            }
            PopupMode::Update => {
                update_song(pool, self.song_id, new_song).await.unwrap();
            }
        }
    }

    pub fn are_any_boxes_editing_mode(&self) -> bool {
        let mut result = false;
        if self.title_box.input_mode == InputMode::Editing {
            result = true;
        } else if self.artist_box.input_mode == InputMode::Editing {
            result = true;
        } else if self.album_box.input_mode == InputMode::Editing {
            result = true;
        } else if self.release_year_box.input_mode == InputMode::Editing {
            result = true;
        } else if self.media_type_box.input_mode == InputMode::Editing {
            result = true;
        }
        result
    }

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

    pub fn set_all_input_modes(&mut self, new_mode: InputMode) {
        self.title_box.input_mode = new_mode.clone();
        self.artist_box.input_mode = new_mode.clone();
        self.album_box.input_mode = new_mode.clone();
        self.release_year_box.input_mode = new_mode.clone();
        self.media_type_box.input_mode = new_mode;
    }
}
impl Widget for Popup {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = match self.mode {
            PopupMode::Create => Title::from(" New Song "),
            PopupMode::Update => Title::from(" Edit Song "),
        };
        let instructions = Title::from(Line::from(vec![
            " Cancel ".into(),
            "<ESC>".yellow().bold(),
            " Next Field ".into(),
            "<Tab>".yellow().bold(),
            " Submit ".into(),
            "<Enter> ".yellow().bold(),
        ]));
        let block = Block::default()
            .borders(Borders::all())
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            );

        let vert_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(18),
                Constraint::Percentage(18),
                Constraint::Percentage(18),
                Constraint::Percentage(18),
            ])
            .split(block.inner(area));
        let horiz_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(vert_layout[3]);

        block.render(area, buf);
        let title_block = Block::default().borders(Borders::ALL);
        let _title_bar = Paragraph::new(Text::from(format!(
            " Title: {}",
            self.title_box.get_input()
        )))
        .left_aligned()
        .block(title_block)
        .render(vert_layout[0], buf);

        let artist_block = Block::default().borders(Borders::ALL);
        let _artist_bar = Paragraph::new(Text::from(format!(
            " Artist: {}",
            self.artist_box.get_input()
        )))
        .left_aligned()
        .block(artist_block)
        .render(vert_layout[1], buf);

        let album_block = Block::default().borders(Borders::ALL);
        let _album_bar = Paragraph::new(Text::from(format!(
            " Album: {}",
            self.album_box.get_input()
        )))
        .left_aligned()
        .block(album_block)
        .render(vert_layout[2], buf);
        let _album_block = Block::default().borders(Borders::ALL);

        let year_block = Block::default().borders(Borders::ALL);
        let _year_bar = Paragraph::new(Text::from(format!(
            " Year: {}",
            self.release_year_box.get_input()
        )))
        .left_aligned()
        .block(year_block)
        .render(horiz_layout[0], buf);

        let media_type_block = Block::default().borders(Borders::ALL);
        let _media_type_bar = Paragraph::new(Text::from(format!(
            " Media: {}",
            self.media_type_box.get_input()
        )))
        .left_aligned()
        .block(media_type_block)
        .render(horiz_layout[1], buf);
        Paragraph::new(Text::from(format!(" {} ", self.title_box.get_input()))).render(area, buf);
    }
}
