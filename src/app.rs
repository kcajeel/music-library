use std::{io, rc::Rc};

use crate::{
    database::{self, get_all_songs},
    song::Song,
    tui,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
use sqlx::{pool, MySqlPool};

// App stores the context information for what action is taking place as well as the database pool
#[derive(Debug)]
pub struct App {
    songs: Vec<Song>,
    pool: MySqlPool,
    selected_row: usize,
    create: bool,
    search: bool,
    update: bool,
    delete: bool,
    esc_mode: bool,
    exit: bool,
}
impl App {
    pub fn new(pool: MySqlPool) -> Self {
        // initial state is everything false
        Self {
            songs: Vec::new(),
            pool,
            selected_row: 0,
            create: false,
            search: false,
            update: false,
            delete: false,
            esc_mode: false,
            exit: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        self.songs = get_all_songs(&self.pool).await.unwrap();
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());

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
            Cell::from(" Title".bold()),
            Cell::from("Artist".bold()),
            Cell::from("Album".bold()),
            Cell::from("Year".bold()),
            Cell::from("Media Type".bold()),
        ]);

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

        let search_bar = Paragraph::new(" Search: ".bold()).left_aligned().block(search_block);

        frame.render_widget(search_bar, get_layout(&frame)[0]);
        frame.render_stateful_widget(table, get_layout(frame)[1], &mut table_state);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_keypress_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_keypress_event(&mut self, key_event: KeyEvent) {
        if !self.esc_mode {
            match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('/') => self.search(),
                KeyCode::Char('n') => self.new_song(),
                KeyCode::Char('e') => self.edit_song(),
                KeyCode::Char('d') => self.delete_song(),
                KeyCode::Up => self.selected_row = (self.selected_row - 1).clamp(0, 10),
                KeyCode::Down => self.selected_row = (self.selected_row + 1).clamp(0, 10),
                _ => {}
            }
        } else {
            match key_event.code {
                KeyCode::Esc => {
                    self.search = false;
                    self.create = false;
                    self.delete = false;
                    self.update = false;
                    self.esc_mode = false;
                },
                _ => {}
            }
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn search(&mut self) {
        self.search = !self.search;
        self.esc_mode = true;
    }

    fn new_song(&mut self) {
        self.create = true;
        self.esc_mode = true;
    }

    fn edit_song(&mut self) {
        self.update = true;
        self.esc_mode = true;
    }

    fn delete_song(&mut self) {
        self.delete = true;
        self.esc_mode = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
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
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let current_state = Text::from(vec![Line::from(vec![
            "Search: ".into(),
            match self.search {
                true => "true".to_owned().yellow(),
                _ => "false".to_owned().yellow(),
            },
        ])]);

        // let current_state = Text::from(vec![Line::from(vec![
        //     "Search: ".into(),
        //     match self.search {
        //         true => "true".to_owned().yellow(),
        //         _ => "false".to_owned().yellow(),
        //     },

        // ])]);
        // // Paragraph::new(current_state)
        // //     .centered()
        // //     .block(block)
        // //     .render(area, buf);
    }
}

fn song_to_row(song: &Song) -> Row {
    Row::new(vec![
        format!(" {}", song.title.clone()),
        song.artist.clone(),
        song.album.clone(),
        song.release_year.to_string(),
        song.media_type.clone(),
    ])
}

fn get_layout(frame: &Frame) -> Rc<[Rect]> {
    let frame_percentage = 90;

    Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Percentage(100 - frame_percentage), 
        Constraint::Percentage(frame_percentage),
    ])
    .split(frame.size())
}
