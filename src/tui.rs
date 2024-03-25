use std::io::{self, stdout, Stdout};

use crossterm::{execute, terminal::*};
use ratatui::prelude::*;

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal
pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

/// Restore the terminal to its original state
pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

// use std::{io, vec};

// use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
// use ratatui::{
//     prelude::*,
//     symbols::border,
//     widgets::{block::*, *},
// };

// mod tui;

// fn main() -> io::Result<()> {
//     let mut terminal = tui::init()?;
//     let app_result = App::default().run(&mut terminal);
//     tui::restore()?;
//     app_result
// }

// #[derive(Debug, Default)]
// pub struct App {
//     exit: bool,
// }

// impl App {
//     /// runs the application's main loop until the user quits
//     pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
//         while !self.exit {
//             terminal.draw(|frame| self.render_frame(frame))?;
//             self.handle_events()?;
//         }
//         Ok(())
//     }

//     fn render_frame(&self, frame: &mut Frame) {

//     let layout = Layout::default()
//     .direction(Direction::Vertical)
//     .constraints(vec![
//         Constraint::Percentage(10),
//         Constraint::Percentage(90),
//     ])
//     .split(frame.size());

        

//             let rows = [
//                 // Row::new(vec![
//                 //     Cell::from(" My Iron Lung"), Cell::from("Radiohead"), Cell::from("The Bends"), Cell::from("1995").yellow().bold(), Cell::from("Digital Download")]), 
//                 Row::new(vec![
//                     Cell::from(" Paranoid"), Cell::from("Black Sabbath"), Cell::from("Paranoid"), Cell::from("1970"), Cell::from("Vinyl").yellow().bold()]), 
//                                     //   Row::new(vec![" ..."]),
//                                       ];
//             // Columns widths are constrained in the same way as Layout...
//             let widths = [
//                 Constraint::Percentage(25),
//                 Constraint::Percentage(25),
//                 Constraint::Percentage(25),
//                 Constraint::Percentage(25),
//                 Constraint::Percentage(25),
//             ];
//             let table = Table::new(rows, widths)
//                 // ...and they can be separated by a fixed spacing.
//                 .column_spacing(1)
//                 // You can set the style of the entire Table.
//                 // It has an optional header, which is simply a Row always visible at the top.
//                 .header(
//                     Row::new(vec![" Song", "Artist", "Album", "Year", "Media Type"])
//                         .style(Style::new().bold())
//                         // To add space between the header and the rest of the rows, specify the margin
//                         .bottom_margin(1),
//                 );

//         // frame.render_widget(
//         //     Paragraph::new("Search").block(Block::new().borders(Borders::all())), layout[0]);
//         // frame.render_widget(
//         //     table, layout[1]);

//             let inner_text = Text::from(vec![Line::from(vec![
//             " Title: ".into(),
//             "Editing the title".yellow().bold(),
//             " Artist: ".into(),
//             "The Beatles".yellow().bold(),
//             " Album: ".into(),
//             "Magical Mystery Tour".yellow().bold(),
//             " Release Year: ".into(),
//             "1967".yellow().bold(),
//             " Media Type: ".into(),
//             "Vinyl".yellow().bold(),
//             ])]);

//             let block = Block::bordered().title(" Enter a New Song ").title_alignment(Alignment::Center).borders(Borders::all());
//             let inner = Paragraph::new(inner_text);

//             let outer_area = layout[1];
//             let inner_area = block.inner(outer_area);
        
            
//         frame.render_widget(block, outer_area);
//         frame.render_widget(inner, inner_area);

//         frame.render_widget(self, frame.size());
//     }

//     /// updates the application's state based on user input
//     fn handle_events(&mut self) -> io::Result<()> {
//         match event::read()? {
//             // it's important to check that the event is a key press event as
//             // crossterm also emits key release and repeat events on Windows.
//             Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
//                 self.handle_key_event(key_event)
//             }
//             _ => {}
//         };
//         Ok(())
//     }

//     fn handle_key_event(&mut self, key_event: KeyEvent) {
//         match key_event.code {
//             KeyCode::Char('q') => self.exit(),
//             KeyCode::Char('/') => self.search(),
//             KeyCode::Char('n') | KeyCode::Char('N') => self.new_song(),
//             KeyCode::Char('e') | KeyCode::Char('E') => self.edit_song(),
//             _ => {}
//         }
//     }

//     fn search(&mut self) {

//     }

//     fn new_song(&mut self) {

//     }

//     fn edit_song(&mut self) {

//     }

//     fn exit(&mut self) {
//         self.exit = true;
//     }

// }

// impl Widget for &App {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let title = Title::from(" Music Library ".bold());
//         let instructions = Title::from(Line::from(vec![

//             // " Search ".into(),
//             // "</>".blue().bold(),
//             // " New Song ".into(),
//             // "<N>".blue().bold(),
//             // " Edit Song ".into(),
//             // "<E>".blue().bold(),
//             // " Quit ".into(),
//             // "<Q> ".blue().bold(),
//             " Enter Song".into(),
//             "<Enter>".blue().bold(),
//             " Go Back".into(),
//             "<Esc> ".blue().bold(),
//             " Quit ".into(),
//             "<Q> ".blue().bold(),
//         ]));
//         let block = Block::default()
//             .title(title.alignment(Alignment::Center))
//             .title(
//                 instructions
//                     .alignment(Alignment::Center)
//                     .position(Position::Bottom),
//             )
//             .borders(Borders::ALL)
//             .border_set(border::THICK);

//         let counter_text = Text::from(
//             ""
//         );

//         Paragraph::new(counter_text)
//             .block(block)
//             .render(area, buf);
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn render() {
//         let app = App::default();
//         let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

//         app.render(buf.area, &mut buf);

//         let mut expected = Buffer::with_lines(vec![
//             "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
//             "┃                    Value: 0                    ┃",
//             "┃                                                ┃",
//             "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
//         ]);
//         let title_style = Style::new().bold();
//         let counter_style = Style::new().yellow();
//         let key_style = Style::new().blue().bold();
//         expected.set_style(Rect::new(14, 0, 22, 1), title_style);
//         expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
//         expected.set_style(Rect::new(13, 3, 6, 1), key_style);
//         expected.set_style(Rect::new(30, 3, 7, 1), key_style);
//         expected.set_style(Rect::new(43, 3, 4, 1), key_style);

//         // note ratatui also has an assert_buffer_eq! macro that can be used to
//         // compare buffers and display the differences in a more readable way
//         assert_eq!(buf, expected);
//     }

//     #[test]
//     fn handle_key_event() -> io::Result<()> {
//         let mut app = App::default();

//         let mut app = App::default();
//         app.handle_key_event(KeyCode::Char('q').into());
//         assert_eq!(app.exit, true);

//         Ok(())
//     }
// }