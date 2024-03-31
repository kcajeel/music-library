use std::io;

use crate::{
    database::{self, get_all_songs},
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
    pool: MySqlPool,
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
            pool,
            create: false,
            search: false,
            update: false,
            delete: false,
            esc_mode: false,
            exit: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind ==KeyEventKind::Press => {
                self.handle_keypress_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_keypress_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('/') => self.search(),
            KeyCode::Char('n') => self.new_song(),
            KeyCode::Char('e') => self.edit_song(),
            KeyCode::Char('d') => self.delete_song(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn search(&mut self) {
        self.search = !self.search;
    }

    fn new_song(&mut self) {
        self.create = true;
    }

    fn edit_song(&mut self) {
        self.update = true;
    }

    fn delete_song(&mut self) {
        self.delete = true;
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
            }
        ])]);

        Paragraph::new(current_state).centered().block(block).render(area, buf);
            
    }
}
