use std::{
    io,
    path::{Path, PathBuf},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
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
    current_dir_path: PathBuf,
    current_dir_contents: Vec<PathBuf>,
    cursor_position: usize,
}

impl App {
    fn new() -> Self {
        let current_dir_contents = std::fs::read_dir("./")
            .unwrap()
            .filter_map(|maybe_dir_entry| {
                let dir_entry = maybe_dir_entry.ok()?;
                Some(dir_entry.path())
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
            KeyCode::Right if self.currently_on_dir() => {
                self.go_into_dir();
            }
            KeyCode::Left => {
                self.go_out_of_dir();
            }
            _ => {}
        }
    }

    fn currently_on_dir(&self) -> bool {
        self.current_dir_contents[self.cursor_position].is_dir()
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

    fn go_into_dir(&mut self) {
        self.current_dir_path
            .push(&self.current_dir_contents[self.cursor_position]);
        self.update_current_dir_contents();
    }

    fn go_out_of_dir(&mut self) {
        self.current_dir_path.pop();
        self.update_current_dir_contents();
    }

    fn update_current_dir_contents(&mut self) {
        self.current_dir_contents = std::fs::read_dir(&self.current_dir_path)
            .unwrap()
            .filter_map(|maybe_dir_entry| {
                let dir_entry = maybe_dir_entry.ok()?;
                Some(dir_entry.path())
            })
            .collect();
    }

    fn get_formatted_path(&self) -> Vec<Line<'static>> {
        self.current_dir_contents
            .iter()
            .enumerate()
            .map(|(index, entity)| Self::format_path(entity, index == self.cursor_position))
            .collect()
    }

    fn format_path(entity: &Path, is_selected: bool) -> Line<'static> {
        let prefix = if is_selected { "> " } else { "  " };

        let name = entity
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("<invalid utf-8>");

        let text = format!("{prefix}{name}");

        if entity.is_dir() {
            Line::from(text).blue()
        } else if entity.is_file() {
            Line::from(text).yellow()
        } else {
            Line::from(text)
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" TUI File Explorer ");

        let lines: Vec<Line> = self.get_formatted_path();

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
