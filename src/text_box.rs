// This module is sourced from https://github.com/ratatui-org/ratatui/blob/main/examples/user_input.rs
// Thank you to joshka from the Ratatui discord server for the recommendation

use ratatui::{style::Stylize, text::Text, widgets::Paragraph};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum InputMode {
    Normal,
    Editing,
}

/// TextBox holds the state of the widget
#[derive(Debug, Clone)]
pub struct TextBox {
    /// Title of the box (displayed before text)
    title: String,
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    cursor_position: usize,
    /// Current input mode
    pub input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
}

impl TextBox {
    pub const fn new(title: String) -> Self {
        Self {
            title,
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            cursor_position: 0,
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    pub fn submit_message(&mut self) {
        self.messages.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }

    pub fn set_input_mode(&mut self, input_mode: InputMode) {
        self.input_mode = input_mode;
    }

    pub fn get_input_mode(&self) -> &InputMode {
        &self.input_mode
    }

    pub fn get_input(&self) -> &String {
        &self.input
    }
    pub fn get_mesages(&self) -> Vec<String> {
        self.messages.clone()
    }

    pub fn get_widget(&self) -> Paragraph {
        match self.input_mode {
            InputMode::Normal => {
                Paragraph::new(Text::from(format!(" {}: {}", self.title, self.input)))
                    .left_aligned()
            }
            InputMode::Editing => Paragraph::new(
                Text::from(format!(" {}: {}", self.title, self.input))
                    .yellow()
                    .bold(),
            )
            .left_aligned(),
        }
    }
}
