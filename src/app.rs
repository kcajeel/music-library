//  This file holds the code that displays the UI and handles input

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
use std::{io, rc::Rc};

// AppMode stores the app's current input mode
#[derive(Debug, PartialEq)]
enum AppMode {
    Normal,
    New,
    Search,
    Edit,
    Delete,
    Exit,
}

// App stores the context information for what action is taking place as well as the database pool
#[derive(Debug)]
pub struct App {
    songs: Vec<Song>, // list of songs to display
    pool: MySqlPool,  // database connection pool

    selected_row: usize, // selected row of the table
    mode: AppMode,       // current mode
    debug: bool,         // set to True to display debug info
    esc_mode: bool,      // in esc_mode, hitting "Esc" returns to Normal mode

    // Widgets to display
    searchbar: TextBox,
    new_popup: Popup,
    edit_popup: Popup,
}
impl App {
    pub fn new(pool: MySqlPool) -> Self {
        // initial state is everything false.
        Self {
            songs: Vec::new(),
            pool,
            selected_row: 1,
            mode: AppMode::Normal,
            debug: false,
            esc_mode: false,
            searchbar: TextBox::new("Search".to_owned()),
            new_popup: Popup::new(PopupMode::New, 0),
            edit_popup: Popup::new(PopupMode::Edit, 0),
        }
    }

    pub async fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        // try to get songs from database, if failure return a template song
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
        // handle events and render the app until the user exits
        while self.mode != AppMode::Exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        // initialize table rows
        let mut rows: Vec<Row> = Vec::new();
        for song in &self.songs {
            let row = song_to_row(song);
            rows.push(row);
        }
        // table title and instructions
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
        // display blocks for the table and searchbar
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
        // store the selected row in the table's state
        let mut table_state: TableState =
            TableState::default().with_selected(Some(self.selected_row.clamp(0, self.songs.len())));
        // Table row headers
        let header = Row::new(vec![
            Cell::from(" Title".bold()),
            Cell::from("Artist".bold()),
            Cell::from("Album".bold()),
            Cell::from("Year".bold()),
            Cell::from("Media Type".bold()),
        ]);
        // Create table
        let table = Table::new(
            rows,
            [
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
        // render searchbar and table
        frame.render_widget(Clear, self.get_layout(&frame)[0]);
        frame.render_widget(
            self.searchbar.get_widget().block(search_block),
            self.get_layout(&frame)[0],
        );
        frame.render_stateful_widget(table, self.get_layout(frame)[1], &mut table_state);

        // if app mode is new, edit, or delete, render an area for the popup
        if self.mode == AppMode::New || self.mode == AppMode::Edit || self.mode == AppMode::Delete {
            let popup_area = centered_rect(frame.size(), 70, 50);
            frame.render_widget(Clear, popup_area);
            // display appropriate popup for the app's mode
            match self.mode {
                AppMode::New => render_popup(frame, self.new_popup.clone(), popup_area),
                AppMode::Edit => render_popup(frame, self.edit_popup.clone(), popup_area),
                AppMode::Delete => render_delete_popup(frame, popup_area),
                _ => {}
            }
        }
        // render debug mode info
        if self.debug {
            frame.render_widget(
            Text::raw(format!(
                "app mode: {:?}, searchbar mode: {:#?}, esc mode: {:#?}, title box: {:?}, title box input mode: {:?}\n
                selected row: {:?}",
                &self.mode,
                self.searchbar.get_input_mode(),
                self.esc_mode,
                self.new_popup.title_box.get_input(),
                self.new_popup.title_box.get_input_mode(),
                self.selected_row,
            )),
            self.get_layout(&frame)[2],
        );
        }
    }

    // Handle keyboard input events
    async fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_keypress_event(key_event).await
            }
            _ => {}
        };
        Ok(())
    }

    // logic for input depending on app's state
    async fn handle_keypress_event(&mut self, key_event: KeyEvent) {
        if !self.esc_mode {
            match key_event.code {
                // if not in esc mode, toggle app mode depending on key pressed
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('/') => self.toggle_search(),
                KeyCode::Char('n') => self.toggle_new_song(),
                KeyCode::Char('e') => self.toggle_edit_song(),
                KeyCode::Char('d') => self.toggle_delete_song(),
                KeyCode::Up | KeyCode::Char('k') => {
                    // scroll up in the table
                    self.selected_row = if self.selected_row == 0 { // wrap around if up is pressed enough times
                        self.songs.len() - 1
                    } else {    // go up otherwise
                        self.selected_row - 1
                    };
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    // scroll down in the table
                    self.selected_row = if self.selected_row + 1 >= self.songs.len() {
                        0
                    } else {
                        self.selected_row + 1
                    };
                }
                _ => {}
            }
        } else {
            match key_event.code {
                // return to Normal state if in esc mode
                KeyCode::Esc => {
                    self.mode = AppMode::Normal;
                    self.esc_mode = false;
                    self.searchbar.set_input_mode(InputMode::Normal);
                    self.new_popup.set_all_input_modes(InputMode::Normal);
                    self.edit_popup.set_all_input_modes(InputMode::Normal);
                }
                _ => {}
            }

            // Perform functions depending on app mode
            match self.mode {
                AppMode::Search => match key_event.code {
                    KeyCode::Char(input_char) => {
                        // enter query as the user types
                        self.searchbar.enter_char(input_char);
                        self.submit_search_query(self.searchbar.get_input().to_string())
                            .await;
                    }
                    KeyCode::Backspace => self.searchbar.delete_char(),
                    KeyCode::Left => self.searchbar.move_cursor_left(),
                    KeyCode::Right => self.searchbar.move_cursor_right(),
                    KeyCode::Enter => {
                        // exit Search mode and clear searchbar if Enter is pressed
                        self.submit_search_query(self.searchbar.get_input().to_string())
                            .await;
                        self.searchbar.submit_message();
                        self.toggle_search();
                    }
                    _ => {}
                },
                AppMode::New => {
                    if self.new_popup.do_all_boxes_have_text() {
                        // check if each box has some input
                        match key_event.code {
                            KeyCode::Enter => {
                                // submit new song and return to Normal mode if enter is pressed
                                self.new_popup.submit(&self.pool).await;
                                self.toggle_new_song();
                                self.new_popup.set_all_input_modes(InputMode::Normal);
                                self.submit_search_query("".to_owned()).await;
                            }
                            _ => {}
                        }
                    }
                    // This code lets the user input characters and edit each box.
                    // It also lets them use "Tab" to cycle through boxes
                    if self.new_popup.title_box.get_input_mode() == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.new_popup.title_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.new_popup.title_box.delete_char(),
                            KeyCode::Left => self.new_popup.title_box.move_cursor_left(),
                            KeyCode::Right => self.new_popup.title_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.new_popup.artist_box.set_input_mode(InputMode::Editing);
                                self.new_popup.title_box.set_input_mode(InputMode::Normal);
                            }
                            _ => {}
                        }
                    } else if self.new_popup.artist_box.get_input_mode() == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.new_popup.artist_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.new_popup.artist_box.delete_char(),
                            KeyCode::Left => self.new_popup.artist_box.move_cursor_left(),
                            KeyCode::Right => self.new_popup.artist_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.new_popup.album_box.set_input_mode(InputMode::Editing);
                                self.new_popup.artist_box.set_input_mode(InputMode::Normal);
                            }
                            _ => {}
                        }
                    } else if self.new_popup.album_box.get_input_mode() == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.new_popup.album_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.new_popup.album_box.delete_char(),
                            KeyCode::Left => self.new_popup.album_box.move_cursor_left(),
                            KeyCode::Right => self.new_popup.album_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.new_popup
                                    .release_year_box
                                    .set_input_mode(InputMode::Editing);
                                self.new_popup.album_box.set_input_mode(InputMode::Normal);
                            }
                            _ => {}
                        }
                    } else if self.new_popup.release_year_box.get_input_mode() == InputMode::Editing
                    {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                if input_char.is_numeric() {
                                    // only lets the user input numbers
                                    self.new_popup.release_year_box.enter_char(input_char);
                                }
                            }
                            KeyCode::Backspace => self.new_popup.release_year_box.delete_char(),
                            KeyCode::Left => self.new_popup.release_year_box.move_cursor_left(),
                            KeyCode::Right => self.new_popup.release_year_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.new_popup
                                    .media_type_box
                                    .set_input_mode(InputMode::Editing);
                                self.new_popup
                                    .release_year_box
                                    .set_input_mode(InputMode::Normal);
                            }
                            KeyCode::Enter => {
                                self.new_popup.release_year_box.submit_message();
                            }
                            _ => {}
                        }
                    } else if self.new_popup.media_type_box.get_input_mode() == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.new_popup.media_type_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.new_popup.media_type_box.delete_char(),
                            KeyCode::Left => self.new_popup.media_type_box.move_cursor_left(),
                            KeyCode::Right => self.new_popup.media_type_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.new_popup.title_box.set_input_mode(InputMode::Editing);
                                self.new_popup
                                    .media_type_box
                                    .set_input_mode(InputMode::Normal);
                            }
                            _ => {}
                        }
                    }
                }
                AppMode::Edit => {
                    if self.edit_popup.do_all_boxes_have_text() {
                        // check if each box has some input
                        match key_event.code {
                            KeyCode::Enter => {
                                // submit edited song and return to Normal mode if enter is pressed
                                self.edit_popup.submit(&self.pool).await;
                                self.toggle_edit_song();
                                self.edit_popup.set_all_input_modes(InputMode::Normal);
                                self.submit_search_query("".to_owned()).await;
                            }
                            _ => {}
                        }
                    }
                    // Same as the code with the New popup
                    if self.edit_popup.title_box.get_input_mode() == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.edit_popup.title_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.edit_popup.title_box.delete_char(),
                            KeyCode::Left => self.edit_popup.title_box.move_cursor_left(),
                            KeyCode::Right => self.edit_popup.title_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.edit_popup
                                    .artist_box
                                    .set_input_mode(InputMode::Editing);
                                self.edit_popup.title_box.set_input_mode(InputMode::Normal);
                            }
                            KeyCode::Enter => {
                                self.edit_popup.title_box.submit_message();
                            }
                            _ => {}
                        }
                    } else if self.edit_popup.artist_box.get_input_mode() == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.edit_popup.artist_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.edit_popup.artist_box.delete_char(),
                            KeyCode::Left => self.edit_popup.artist_box.move_cursor_left(),
                            KeyCode::Right => self.edit_popup.artist_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.edit_popup.album_box.set_input_mode(InputMode::Editing);
                                self.edit_popup.artist_box.set_input_mode(InputMode::Normal);
                            }
                            KeyCode::Enter => {
                                self.edit_popup.artist_box.submit_message();
                            }
                            _ => {}
                        }
                    } else if self.edit_popup.album_box.get_input_mode() == InputMode::Editing {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.edit_popup.album_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.edit_popup.album_box.delete_char(),
                            KeyCode::Left => self.edit_popup.album_box.move_cursor_left(),
                            KeyCode::Right => self.edit_popup.album_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.edit_popup
                                    .release_year_box
                                    .set_input_mode(InputMode::Editing);
                                self.edit_popup.album_box.set_input_mode(InputMode::Normal);
                            }
                            KeyCode::Enter => {
                                self.edit_popup.album_box.submit_message();
                            }
                            _ => {}
                        }
                    } else if self.edit_popup.release_year_box.get_input_mode()
                        == InputMode::Editing
                    {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                if input_char.is_numeric() {
                                    // only lets the user input numbers
                                    self.edit_popup.release_year_box.enter_char(input_char);
                                }
                            }
                            KeyCode::Backspace => self.edit_popup.release_year_box.delete_char(),
                            KeyCode::Left => self.edit_popup.release_year_box.move_cursor_left(),
                            KeyCode::Right => self.edit_popup.release_year_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.edit_popup
                                    .media_type_box
                                    .set_input_mode(InputMode::Editing);
                                self.edit_popup
                                    .release_year_box
                                    .set_input_mode(InputMode::Normal);
                            }
                            KeyCode::Enter => {
                                self.edit_popup.release_year_box.submit_message();
                            }
                            _ => {}
                        }
                    } else if self.edit_popup.media_type_box.get_input_mode() == InputMode::Editing
                    {
                        match key_event.code {
                            KeyCode::Char(input_char) => {
                                self.edit_popup.media_type_box.enter_char(input_char);
                            }
                            KeyCode::Backspace => self.edit_popup.media_type_box.delete_char(),
                            KeyCode::Left => self.edit_popup.media_type_box.move_cursor_left(),
                            KeyCode::Right => self.edit_popup.media_type_box.move_cursor_right(),
                            KeyCode::Tab => {
                                self.edit_popup.title_box.set_input_mode(InputMode::Editing);
                                self.edit_popup
                                    .media_type_box
                                    .set_input_mode(InputMode::Normal);
                            }
                            KeyCode::Enter => {
                                self.edit_popup.media_type_box.submit_message();
                            }
                            _ => {}
                        }
                    }
                }
                AppMode::Delete => match key_event.code {
                    KeyCode::Char('Y') => {
                        // delete song and return to Normal mode if capital Y is pressed
                        let selected_song = self.get_selected_song();
                        self.purge_song(selected_song).await;
                        self.toggle_delete_song();
                        self.submit_search_query("".to_owned()).await;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    // returns the layout for the app
    fn get_layout(&self, frame: &Frame) -> Rc<[Rect]> {
        let frame_percentage = if self.debug { 70 } else { 90 };
        // layout constraints if debug
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
            // non-debug layout
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
        // turn exit mode on (it never needs to turn off)
        self.mode = AppMode::Exit;
    }

    fn toggle_search(&mut self) {
        // change mode to Normal if Search, vice versa
        self.mode = if self.mode == AppMode::Normal {
            AppMode::Search
        } else {
            AppMode::Normal
        };
        self.esc_mode = !self.esc_mode; // invert esc_mode

        if self.searchbar.get_input_mode() != InputMode::Editing {
            // invert searchbar edit mode
            self.searchbar.set_input_mode(InputMode::Editing);
        } else {
            self.searchbar.set_input_mode(InputMode::Normal);
        }
    }

    fn toggle_new_song(&mut self) {
        self.mode = if self.mode == AppMode::Normal {
            // change mode to Normal or New depending on current mode
            AppMode::New
        } else {
            AppMode::Normal
        };
        self.esc_mode = !self.esc_mode; // invert esc_mode
        if self.mode == AppMode::New {
            // set title box to edit mode if appmode is New
            if !self.new_popup.are_any_boxes_editing_mode() {
                self.new_popup.title_box.set_input_mode(InputMode::Editing);
            }
        }
    }

    fn toggle_edit_song(&mut self) {
        self.mode = if self.mode == AppMode::Normal {
            AppMode::Edit // change mode to Normal if Edit, change to Edit if Normal
        } else {
            AppMode::Normal
        };
        self.esc_mode = !self.esc_mode; // invert esc_mode

        self.edit_popup.clear_all_boxes(); // clear textboxes and populate with selected song info
        if self.mode == AppMode::Edit {
            let selected_song = &self.get_selected_song();
            self.edit_popup.populate_textboxes_with_song(selected_song);

            if !self.edit_popup.are_any_boxes_editing_mode() {
                // set title box mode to Editing
                self.edit_popup.title_box.set_input_mode(InputMode::Editing);
            }
        }
    }

    fn toggle_delete_song(&mut self) {
        // Commenting this function would be redundant
        self.mode = if self.mode == AppMode::Normal {
            AppMode::Delete
        } else {
            AppMode::Normal
        };
        self.esc_mode = !self.esc_mode;
    }

    async fn submit_search_query(&mut self, query: String) {
        // submit query, if it fails print an error
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

    // correlates the selected_row to a song ID, returns song
    fn get_selected_song(&mut self) -> Song {
        let mut ids = Vec::new(); // create ids vec

        for song in &self.songs {
            // add song id's to ids vec
            ids.push(song.id);
        }
        // get selected song id using index of selected row
        if let Some(selected_id) = ids.get(self.selected_row) {
            for song in &self.songs {
                // return the song matching the id
                if song.id == *selected_id {
                    return song.clone();
                }
            }
        }
        // if no songs in ids, there are no songs in database.
        // print error, return to New mode so the user can add a song
        eprintln!("No songs in database");
        self.mode = AppMode::New;
        Song::default() // lol
    }

    // delete the song and print any errors
    async fn purge_song(&mut self, song: Song) {
        match delete_song(&self.pool, song.id).await {
            Ok(_) => {}
            Err(error) => {
                eprintln!("Error deleting songs: {}", error)
            }
        };
    }
}

fn song_to_row(song: &Song) -> Row {
    // add a space to the start of the title
    let mut title = song.title.clone();
    title.insert(0, ' ');
    // create a row from the song's fields
    Row::new(vec![
        title,
        song.artist.clone(),
        song.album.clone(),
        song.release_year.to_string(),
        song.media_type.clone(),
    ])
}

// center a rectangular area and return it given x and y percentages
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

// render fn for popups
fn render_popup(frame: &mut Frame, menu: Popup, area: Rect)
where
    Popup: Sized,
{
    // title and instructions for popup box
    let title = match menu.get_popup_mode() {
        PopupMode::New => Title::from(" New Song "),
        PopupMode::Edit => Title::from(" Edit Song "),
    };
    let instructions = Title::from(Line::from(vec![
        " Cancel ".into(),
        "<ESC>".yellow().bold(),
        " Next Field ".into(),
        "<Tab>".yellow().bold(),
        " Submit ".into(),
        "<Enter> ".yellow().bold(),
    ]));
    // display block for popup boxes
    let popup_block = Block::default()
        .borders(Borders::all())
        .title(title.alignment(Alignment::Center))
        .title(
            instructions
                .alignment(Alignment::Center)
                .position(Position::Bottom),
        );

    // Divide the area horizontally and vertically for displaying text boxes
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

    frame.render_widget(popup_block, area); // render popup block

    // render text boxes
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

// render function for the delete popup
fn render_delete_popup(frame: &mut Frame, area: Rect) {
    // instructions for the delete block
    let delete_instructions = Title::from(Line::from(vec![
        " Cancel".into(),
        "<ESC>".yellow().bold(),
        " Yes".into(),
        "<Y> ".yellow().bold(),
    ]));
    // delete block
    let delete_block = Block::default()
        .borders(Borders::all())
        .title(" Delete Song ")
        .title_alignment(Alignment::Center)
        .title(
            delete_instructions
                .alignment(Alignment::Center)
                .position(Position::Bottom),
        );
    // render dleete popup
    frame.render_widget(
        Paragraph::new(
            Text::from(" Are you sure you want to delete this song? ")
                .bold()
                .red()
                .rapid_blink()
                .alignment(Alignment::Center),
        )
        .block(delete_block),
        area,
    );
}
