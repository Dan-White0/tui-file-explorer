use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug, Default)]
struct App {
    exit: bool,
    current_dir_contents: Vec<String>,
    cursor_position: usize,
}

impl App {
    fn new() -> Self {
        let current_dir_contents = std::fs::read_dir("./")
            .unwrap()
            .filter_map(|maybe_dir_entry| {
                let dir_entry = maybe_dir_entry.ok()?;
                let path_buf = dir_entry.path();
                let file_name = path_buf.file_name()?;
                let string = file_name.to_str()?;
                Some(string.to_string())
            })
            .collect();

        App {
            current_dir_contents,
            ..Default::default()
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // Blocks until an event is read
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Down => {
                self.move_cursor_down();
            }
            KeyCode::Up => {
                self.move_cursor_up();
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn move_cursor_down(&mut self) {
        if self.cursor_position == self.current_dir_contents.len() - 1 {
            self.cursor_position = 0;
        } else {
            self.cursor_position += 1;
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_position == 0 {
            self.cursor_position = self.current_dir_contents.len() - 1;
        } else {
            self.cursor_position -= 1;
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" TUI File Explorer");

        let mut lines = Vec::new();
        for (index, entity) in self.current_dir_contents.iter().enumerate() {
            let prefix = if index == self.cursor_position {
                "> "
            } else {
                "  "
            };
            lines.push(Line::from(format!("{}{}", prefix, entity.clone())));
        }

        let text = Text::from(lines);

        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        Paragraph::new(text)
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}
