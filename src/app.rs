use std::{io, rc::Rc};

use crate::{
    database::{delete_song, get_all_songs, get_songs_matching},
    popup::{Popup, PopupMode},
    song::Song,
    text_box::{InputMode, TextBox},
    tui,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{
        block::{Position, Title},
        Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState,
    },
};
use sqlx::MySqlPool;

// AppMode stores the app's current input mode
#[derive(Debug, PartialEq)]
enum AppMode {
    Normal,
    Create,
    Search,
    Update,
    Delete,
    Exit,
}

// App stores the context information for what action is taking place as well as the database pool
#[derive(Debug)]
pub struct App {
    songs: Vec<Song>,
    pool: MySqlPool,

    selected_row: usize,
    mode: AppMode,
    debug: bool,
    esc_mode: bool,

    searchbar: TextBox,
    create_popup: Popup,
    update_popup: Popup,
}
impl App {
    pub fn new(pool: MySqlPool) -> Self {
        // initial state is everything false
        Self {
            songs: Vec::new(),
            pool,
            selected_row: 1,
            mode: AppMode::Normal,
            debug: true,
            esc_mode: false,
            searchbar: TextBox::new("Search".to_owned()),
            create_popup: Popup::new(PopupMode::Create, 0),
            update_popup: Popup::new(PopupMode::Update, 0),
        }
    }

    pub async fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        self.songs = match get_all_songs(&self.pool).await {
            Ok(songs) => songs,
            Err(error) => {
                eprintln!("Error getting songs from database: {}", error);
                vec![Song::new(
                    404,
                    " Error",
                    "Displaying",
                    "Songs. ",
                    404,
                    "Error ",
                )]
            }
        };
        while self.mode != AppMode::Exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        let mut rows: Vec<Row> = Vec::new();
        for song in &self.songs {
            let row = song_to_row(song);
            rows.push(row);
        }

        let title = Title::from(" Music Library ".bold());
        let instructions = Title::from(Line::from(vec![
            " Search ".into(),
            "</>".yellow().bold(),
            " New Song ".into(),
            "<N>".yellow().bold(),
            " Edit Song ".into(),
            "<E>".yellow().bold(),
            " Delete Song ".into(),
            "<D> ".yellow().bold(),
        ]));
        let table_block = Block::default()
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let search_block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let mut table_state: TableState =
            TableState::default().with_selected(Some(self.selected_row.clamp(0, 100)));

        let header = Row::new(vec![
            Cell::from(" ID".bold()),
            Cell::from("Title".bold()),
            Cell::from("Artist".bold()),
            Cell::from("Album".bold()),
            Cell::from("Year".bold()),
            Cell::from("Media Type".bold()),
        ]);

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(30),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
            ],
        )
        .column_spacing(1)
        .header(header)
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>")
        .block(table_block);

        frame.render_widget(Clear, self.get_layout(&frame)[0]);
        frame.render_widget(
            self.searchbar.get_widget().block(search_block),
            self.get_layout(&frame)[0],
        );
        frame.render_stateful_widget(table, self.get_layout(frame)[1], &mut table_state);

        if self.mode == AppMode::Create
            || self.mode == AppMode::Update
            || self.mode == AppMode::Delete
        {
            let popup_area = centered_rect(frame.size(), 50, 50);
            frame.render_widget(Clear, popup_area);

            match self.mode {
                AppMode::Create => render_popup(frame, self.create_popup.clone(), popup_area),
                AppMode::Update => render_popup(frame, self.update_popup.clone(), popup_area),
                AppMode::Delete => render_delete_popup(frame, popup_area),
                _ => {}
            }
        }
        if self.debug {
            frame.render_widget(
            Text::raw(format!(
                "app mode: {:?}, searchbar mode: {:#?}, esc mode: {:#?}, title box: {:?}, title box input mode: {:?}\n
                selected row: {:?}",
                &self.mode,
                self.searchbar.get_input_mode(),
                self.esc_mode,
                self.create_popup.title_box.get_input(),
                self.create_popup.title_box.input_mode,
                self.selected_row,
            )),
            self.get_layout(&frame)[2],
        );
        }
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_keypress_event(key_event).await
            }
            _ => {}
        };
        Ok(())
    }

    async fn handle_keypress_event(&mut self, key_event: KeyEvent) {
        if !self.esc_mode {
            match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('/') => self.toggle_search(),
                KeyCode::Char('n') => self.toggle_new_song(),
                KeyCode::Char('e') => self.toggle_edit_song(),
                KeyCode::Char('d') => self.toggle_delete_song(),
                KeyCode::Up | KeyCode::Char('k') => {
                    self.selected_row = (self.selected_row - 1).clamp(0, 10)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.selected_row = (self.selected_row + 1).clamp(0, 10)
                }
                _ => {}
            }
        } else {
            match key_event.code {
                KeyCode::Esc => {
                    self.mode = AppMode::Normal;
                    self.esc_mode = false;
                    self.searchbar.set_input_mode(InputMode::Normal);
                    self.create_popup.set_all_input_modes(InputMode::Normal);
                }
                _ => {}
            }

            match self.mode {
                AppMode::Search => match key_event.code {
                    KeyCode::Char(input_char) => {
                        self.searchbar.enter_char(input_char);
                        self.submit_search_query(self.searchbar.get_input().to_string())
                            .await;
                    }
                    KeyCode::Backspace => self.searchbar.delete_char(),
                    KeyCode::Left => self.searchbar.move_cursor_left(),
                    KeyCode::Right => self.searchbar.move_cursor_right(),
                    KeyCode::Enter => {
                        self.submit_search_query(self.searchbar.get_input().to_string())
                            .await;
                        self.searchbar.submit_message();
                    }
                    _ => {}
                },
                AppMode::Create => {
                    if self.create_popup.do_all_boxes_have_text() {
                        match key_event.code {
                            KeyCode::Enter => {
                                if self.create_popup.do_all_boxes_have_text() {
                                    self.create_popup.submit(&self.pool).await;
                                }
                            }
                            _ => {}
                        }
                    }
                    if self.create_popup.title_box.input_mode == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.create_popup.title_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.create_popup.title_box.delete_char(),
                            KeyCode::Left => self.create_popup.title_box.move_cursor_left(),
                            KeyCode::Right => self.create_popup.title_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.create_popup.artist_box.input_mode = InputMode::Editing;
                                self.create_popup.title_box.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        }
                    } else if self.create_popup.artist_box.input_mode == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.create_popup.artist_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.create_popup.artist_box.delete_char(),
                            KeyCode::Left => self.create_popup.artist_box.move_cursor_left(),
                            KeyCode::Right => self.create_popup.artist_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.create_popup.album_box.input_mode = InputMode::Editing;
                                self.create_popup.artist_box.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        }
                    } else if self.create_popup.album_box.input_mode == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.create_popup.album_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.create_popup.album_box.delete_char(),
                            KeyCode::Left => self.create_popup.album_box.move_cursor_left(),
                            KeyCode::Right => self.create_popup.album_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.create_popup.release_year_box.input_mode = InputMode::Editing;
                                self.create_popup.album_box.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        }
                    } else if self.create_popup.release_year_box.input_mode == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.create_popup.release_year_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.create_popup.release_year_box.delete_char(),
                            KeyCode::Left => self.create_popup.release_year_box.move_cursor_left(),
                            KeyCode::Right => {
                                self.create_popup.release_year_box.move_cursor_right()
                            }
                            KeyCode::Tab => {
                                self.create_popup.media_type_box.input_mode = InputMode::Editing;
                                self.create_popup.release_year_box.input_mode = InputMode::Normal;
                            }
                            KeyCode::Enter => {
                                self.create_popup.release_year_box.submit_message();
                            }
                            _ => {}
                        }
                    } else if self.create_popup.media_type_box.input_mode == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.create_popup.media_type_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.create_popup.media_type_box.delete_char(),
                            KeyCode::Left => self.create_popup.media_type_box.move_cursor_left(),
                            KeyCode::Right => self.create_popup.media_type_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.create_popup.title_box.input_mode = InputMode::Editing;
                                self.create_popup.media_type_box.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        }
                    }
                }
                AppMode::Update => {
                    if self.update_popup.title_box.input_mode == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.update_popup.title_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.update_popup.title_box.delete_char(),
                            KeyCode::Left => self.update_popup.title_box.move_cursor_left(),
                            KeyCode::Right => self.update_popup.title_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.update_popup.artist_box.input_mode = InputMode::Editing;
                                self.update_popup.title_box.input_mode = InputMode::Normal;
                            }
                            KeyCode::Enter => {
                                self.update_popup.title_box.submit_message();
                            }
                            _ => {}
                        }
                    } else if self.update_popup.artist_box.input_mode == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.update_popup.artist_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.update_popup.artist_box.delete_char(),
                            KeyCode::Left => self.update_popup.artist_box.move_cursor_left(),
                            KeyCode::Right => self.update_popup.artist_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.update_popup.album_box.input_mode = InputMode::Editing;
                                self.update_popup.artist_box.input_mode = InputMode::Normal;
                            }
                            KeyCode::Enter => {
                                self.update_popup.artist_box.submit_message();
                            }
                            _ => {}
                        }
                    } else if self.update_popup.album_box.input_mode == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.update_popup.album_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.update_popup.album_box.delete_char(),
                            KeyCode::Left => self.update_popup.album_box.move_cursor_left(),
                            KeyCode::Right => self.update_popup.album_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.update_popup.release_year_box.input_mode = InputMode::Editing;
                                self.update_popup.album_box.input_mode = InputMode::Normal;
                            }
                            KeyCode::Enter => {
                                self.update_popup.album_box.submit_message();
                            }
                            _ => {}
                        }
                    } else if self.update_popup.release_year_box.input_mode == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.update_popup.release_year_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.update_popup.release_year_box.delete_char(),
                            KeyCode::Left => self.update_popup.release_year_box.move_cursor_left(),
                            KeyCode::Right => {
                                self.update_popup.release_year_box.move_cursor_right()
                            }
                            KeyCode::Tab => {
                                self.update_popup.media_type_box.input_mode = InputMode::Editing;
                                self.update_popup.release_year_box.input_mode = InputMode::Normal;
                            }
                            KeyCode::Enter => {
                                self.update_popup.release_year_box.submit_message();
                            }
                            _ => {}
                        }
                    } else if self.update_popup.media_type_box.input_mode == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.update_popup.media_type_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.update_popup.media_type_box.delete_char(),
                            KeyCode::Left => self.update_popup.media_type_box.move_cursor_left(),
                            KeyCode::Right => self.update_popup.media_type_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.update_popup.title_box.input_mode = InputMode::Editing;
                                self.update_popup.media_type_box.input_mode = InputMode::Normal;
                            }
                            KeyCode::Enter => {
                                self.update_popup.media_type_box.submit_message();
                            }
                            _ => {}
                        }
                    } else {
                        if self.update_popup.do_all_boxes_have_text() {
                            match key_event.code {
                                KeyCode::Enter => {
                                    self.update_popup.submit(&self.pool).await;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                AppMode::Delete => match key_event.code {
                    KeyCode::Char('Y') => {
                        match delete_song(&self.pool, self.selected_row as u32).await {
                            Ok(_) => {}
                            Err(error) => {
                                eprintln!("Error deleting songs: {}", error)
                            }
                        };
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn get_layout(&self, frame: &Frame) -> Rc<[Rect]> {
        let frame_percentage = if self.debug { 70 } else { 90 };

        if self.debug {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(80 - frame_percentage),
                    Constraint::Percentage(frame_percentage),
                    Constraint::Percentage(20),
                ])
                .split(frame.size())
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(100 - frame_percentage),
                    Constraint::Percentage(frame_percentage),
                ])
                .split(frame.size())
        }
    }

    fn exit(&mut self) {
        self.mode = AppMode::Exit;
    }

    fn toggle_search(&mut self) {
        self.mode = if self.mode == AppMode::Normal {
            AppMode::Search
        } else {
            AppMode::Normal
        };
        self.esc_mode = true;
        if *self.searchbar.get_input_mode() != InputMode::Editing {
            self.searchbar.set_input_mode(InputMode::Editing);
        } else {
            self.searchbar.set_input_mode(InputMode::Normal);
        }
    }

    fn toggle_new_song(&mut self) {
        self.mode = if self.mode == AppMode::Normal {
            AppMode::Create
        } else {
            AppMode::Normal
        };
        self.esc_mode = true;
        if !self.create_popup.are_any_boxes_editing_mode() {
            self.create_popup.title_box.input_mode = InputMode::Editing;
        }
    }

    fn toggle_edit_song(&mut self) {
        self.mode = if self.mode == AppMode::Normal {
            AppMode::Update
        } else {
            AppMode::Normal
        };
        self.esc_mode = true;
        if !self.update_popup.are_any_boxes_editing_mode() {
            self.update_popup.title_box.input_mode = InputMode::Editing;
        }
    }

    fn toggle_delete_song(&mut self) {
        self.mode = if self.mode == AppMode::Normal {
            AppMode::Delete
        } else {
            AppMode::Normal
        };
        self.esc_mode = true;
    }

    async fn submit_search_query(&mut self, query: String) {
        self.songs = match get_songs_matching(&self.pool, query).await {
            Ok(songs) => songs,
            Err(error) => {
                eprintln!("Error searching for songs: {}", error);
                vec![Song::new(
                    500,
                    " Error",
                    "Searching",
                    "Songs. ",
                    500,
                    "Error ",
                )]
            }
        };
    }
}

fn song_to_row(song: &Song) -> Row {
    Row::new(vec![
        format!(" {}", song.id.to_string()),
        song.title.clone(),
        song.artist.clone(),
        song.album.clone(),
        song.release_year.to_string(),
        song.media_type.clone(),
    ])
}

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_popup(frame: &mut Frame, menu: Popup, area: Rect)
where
    Popup: Sized,
{
    let title = match menu.mode {
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
    let popup_block = Block::default()
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
        .split(popup_block.inner(area));

    let horiz_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vert_layout[3]);

    frame.render_widget(popup_block, area);

    let title_block = Block::default().borders(Borders::ALL);
    frame.render_widget(
        menu.title_box.get_widget().block(title_block),
        vert_layout[0],
    );

    let artist_block = Block::default().borders(Borders::ALL);
    frame.render_widget(
        menu.artist_box.get_widget().block(artist_block),
        vert_layout[1],
    );

    let album_block = Block::default().borders(Borders::ALL);
    frame.render_widget(
        menu.album_box.get_widget().block(album_block),
        vert_layout[2],
    );

    let year_block = Block::default().borders(Borders::ALL);
    frame.render_widget(
        menu.release_year_box.get_widget().block(year_block),
        horiz_layout[0],
    );

    let media_type_block = Block::default().borders(Borders::ALL);
    frame.render_widget(
        menu.media_type_box.get_widget().block(media_type_block),
        horiz_layout[1],
    );
}

fn render_delete_popup(frame: &mut Frame, area: Rect) {
    let delete_instructions = Title::from(Line::from(vec![
        " Cancel".into(),
        "<ESC>".yellow().bold(),
        " Yes".into(),
        "<Y> ".yellow().bold(),
    ]));

    let delete_block = Block::default()
        .borders(Borders::all())
        .title(" Delete Song ")
        .title_alignment(Alignment::Center)
        .title(
            delete_instructions
                .alignment(Alignment::Center)
                .position(Position::Bottom),
        );
    frame.render_widget(
        Paragraph::new(
            Text::from(" Are you sure you want to delete this song? ")
                .bold()
                .yellow()
                .alignment(Alignment::Center),
        )
        .block(delete_block),
        area,
    );
}
