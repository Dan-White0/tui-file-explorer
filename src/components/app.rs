use itertools::sorted;
use std::{io, path::PathBuf};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

use super::directory_view::get_formatted_paths;

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    current_dir_path: PathBuf,
    current_dir_contents: Vec<PathBuf>,
    cursor_positions: Vec<usize>,
    current_cursor_depth: usize,
}

impl App {
    pub fn new(current_dir_path: PathBuf) -> Self {
        let current_dir_contents = sorted(
            std::fs::read_dir(&current_dir_path)
                .unwrap()
                .filter_map(|maybe_dir_entry| {
                    let dir_entry = maybe_dir_entry.ok()?;
                    Some(dir_entry.path())
                }),
        )
        .collect();

        let current_cursor_depth = current_dir_path.ancestors().count() - 1;
        let cursor_positions = vec![0; current_cursor_depth + 1];

        App {
            current_dir_contents,
            current_dir_path,
            cursor_positions,
            current_cursor_depth,
            ..Default::default()
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
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
        self.current_dir_contents[self.current_cursor_position()].is_dir()
    }

    fn current_cursor_position(&self) -> usize {
        self.cursor_positions[self.current_cursor_depth]
    }

    fn current_cursor_column_and_row(&self, column_height: usize) -> (usize, usize) {
        let current_cursor_pos = self.cursor_positions[self.current_cursor_depth];
        (
            current_cursor_pos / column_height,
            current_cursor_pos % column_height,
        )
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn move_cursor_down(&mut self) {
        if self.current_cursor_position() == self.current_dir_contents.len() - 1 {
            self.cursor_positions[self.current_cursor_depth] = 0;
        } else {
            self.cursor_positions[self.current_cursor_depth] += 1;
        }
    }

    fn move_cursor_up(&mut self) {
        if self.current_cursor_position() == 0 {
            self.cursor_positions[self.current_cursor_depth] = self.current_dir_contents.len() - 1;
        } else {
            self.cursor_positions[self.current_cursor_depth] -= 1;
        }
    }

    fn go_into_dir(&mut self) {
        self.current_dir_path
            .push(&self.current_dir_contents[self.current_cursor_position()]);
        self.update_current_dir_contents();
        self.current_cursor_depth += 1;
        if self.current_cursor_depth >= self.cursor_positions.len() {
            self.cursor_positions.push(0);
        }
    }

    fn go_out_of_dir(&mut self) {
        self.current_dir_path.pop();
        self.update_current_dir_contents();
        self.current_cursor_depth -= 1;
        self.cursor_positions.pop();
    }

    fn update_current_dir_contents(&mut self) {
        self.current_dir_contents = sorted(
            std::fs::read_dir(&self.current_dir_path)
                .unwrap()
                .filter_map(|maybe_dir_entry| {
                    let dir_entry = maybe_dir_entry.ok()?;
                    Some(dir_entry.path())
                }),
        )
        .collect();
    }

    fn get_dir_contents_as_columns(&self, column_height: u16) -> Vec<Vec<PathBuf>> {
        self.current_dir_contents
            .chunks(column_height as usize)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" TUI File Explorer ".bold());
        let dir_line = Line::from(self.current_dir_path.to_str().unwrap());

        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        Paragraph::new(dir_line)
            .left_aligned()
            .block(block)
            .render(area, buf);

        // Height of window, take away 2 for the border and 1 for the current dir
        let column_height = area.height.saturating_sub(3);
        let dir_contents_columns = self.get_dir_contents_as_columns(column_height);

        let dir_contents_area = Rect {
            x: area.x + 1,
            y: area.y + 2,
            width: area.width,
            height: column_height,
        };

        let column_widths: Vec<Constraint> = dir_contents_columns
            .iter()
            .map(|column| {
                Constraint::Length(
                    (column
                        .iter()
                        .map(|e| e.file_name().unwrap().to_str().unwrap().len())
                        .max()
                        .unwrap()
                        + 8) as u16,
                )
            })
            .collect();

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(column_widths)
            .split(dir_contents_area);

        let (cursor_column_index, cursor_row_index) =
            self.current_cursor_column_and_row(column_height as usize);

        for (column_index, (column_area, column_contents)) in
            columns.iter().zip(dir_contents_columns.iter()).enumerate()
        {
            if column_index == cursor_column_index {
                Paragraph::new(Text::from(get_formatted_paths(
                    column_contents,
                    Some(cursor_row_index),
                )))
                .left_aligned()
                .render(*column_area, buf);
            } else {
                Paragraph::new(Text::from(get_formatted_paths(column_contents, None)))
                    .left_aligned()
                    .render(*column_area, buf);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::{File, create_dir};

    use ratatui::style::Style;
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn can_exit() {
        let mut app = App::default();
        assert!(!app.exit);

        app.handle_key_event(KeyCode::Char('q').into());
        assert!(app.exit);
    }

    #[test]
    fn can_move_cursor() {
        let mut app = App {
            current_dir_contents: vec![PathBuf::from("a"), PathBuf::from("b"), PathBuf::from("c")],
            current_dir_path: PathBuf::from("./"),
            cursor_positions: vec![0],
            ..Default::default()
        };

        assert_eq!(app.current_cursor_position(), 0);

        app.handle_key_event(KeyCode::Down.into());
        assert_eq!(app.current_cursor_position(), 1);

        app.handle_key_event(KeyCode::Up.into());
        assert_eq!(app.current_cursor_position(), 0);
    }

    #[test]
    fn can_cursor_wraps_around() {
        let mut app = App {
            current_dir_contents: vec![PathBuf::from("a"), PathBuf::from("b"), PathBuf::from("c")],
            current_dir_path: PathBuf::from("./"),
            cursor_positions: vec![0],
            ..Default::default()
        };

        assert_eq!(app.current_cursor_position(), 0);

        app.handle_key_event(KeyCode::Up.into());
        assert_eq!(app.current_cursor_position(), 2);

        app.handle_key_event(KeyCode::Down.into());
        assert_eq!(app.current_cursor_position(), 0);
    }

    #[test]
    fn can_enter_dir() {
        let tmp_dir = TempDir::new("tmp_dir").unwrap();
        let nested_dir_path =
            PathBuf::from(format!("{}/nested_dir", tmp_dir.path().to_str().unwrap()));
        let _nested_dir = create_dir(&nested_dir_path);
        let file_path = tmp_dir.path().join("file.txt");
        let _tmp_file = File::create(&file_path).unwrap();

        let mut app = App::new(tmp_dir.path().to_path_buf());

        assert_eq!(app.current_dir_path, tmp_dir.path().to_path_buf());
        assert_eq!(
            app.current_dir_contents,
            vec![file_path.clone(), nested_dir_path.clone()]
        );
        assert_eq!(app.current_cursor_position(), 0);

        // Current dir does not change when attempting to enter file
        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.current_dir_path, tmp_dir.path().to_path_buf());
        assert_eq!(
            app.current_dir_contents,
            vec![file_path.clone(), nested_dir_path.clone()]
        );
        assert_eq!(app.current_cursor_position(), 0);

        // But does change if entering dir
        app.handle_key_event(KeyCode::Down.into());
        assert_eq!(app.current_cursor_position(), 1);
        assert_eq!(app.current_dir_path, tmp_dir.path().to_path_buf());
        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.current_cursor_position(), 0);
        assert_eq!(app.current_dir_path, nested_dir_path);
    }

    #[test]
    fn can_exit_dir() {
        let tmp_dir = TempDir::new("tmp_dir").unwrap();
        let file_path = tmp_dir.path().join("file.txt");
        let _tmp_file = File::create(&file_path).unwrap();
        let nested_dir_path =
            PathBuf::from(format!("{}/nested_dir", tmp_dir.path().to_str().unwrap()));
        let _nested_dir = create_dir(&nested_dir_path);
        let nested_file_path_0 = nested_dir_path.join("file_a.txt");
        let nested_file_path_1 = nested_dir_path.join("file_b.txt");
        let _nested_file_0 = File::create(&nested_file_path_0).unwrap();
        let _nested_file_1 = File::create(&nested_file_path_1).unwrap();

        let mut app = App::new(nested_dir_path.clone());
        assert_eq!(app.current_dir_path, nested_dir_path);
        assert_eq!(
            app.current_dir_contents,
            vec![nested_file_path_0.clone(), nested_file_path_1.clone()]
        );
        assert_eq!(app.current_cursor_position(), 0);

        // Go up a dir when left key pressed
        app.handle_key_event(KeyCode::Down.into());
        assert_eq!(app.current_dir_path, nested_dir_path);
        assert_eq!(
            app.current_dir_contents,
            vec![nested_file_path_0.clone(), nested_file_path_1.clone()]
        );
        assert_eq!(app.current_cursor_position(), 1);

        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.current_cursor_position(), 0);
        assert_eq!(
            app.current_dir_contents,
            vec![file_path.clone(), nested_dir_path.clone()]
        );
    }

    #[test]
    fn cursor_position_retained_after_entering_then_exiting_dir() {
        let tmp_dir = TempDir::new("tmp_dir").unwrap();
        let nested_dir_path =
            PathBuf::from(format!("{}/nested_dir", tmp_dir.path().to_str().unwrap()));
        let _nested_dir = create_dir(&nested_dir_path);
        let file_path = tmp_dir.path().join("file.txt");
        let _tmp_file = File::create(&file_path).unwrap();

        let mut app = App::new(tmp_dir.path().to_path_buf());

        assert_eq!(app.current_dir_path, tmp_dir.path().to_path_buf());
        assert_eq!(app.current_cursor_position(), 0);

        // Change cursor position to 1
        app.handle_key_event(KeyCode::Down.into());
        assert_eq!(app.current_cursor_position(), 1);
        assert_eq!(app.current_dir_path, tmp_dir.path().to_path_buf());

        // Entering directory sets cursor position to 0, as this is the first time entering
        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.current_cursor_position(), 0);
        assert_eq!(app.current_dir_path, nested_dir_path);

        // Exiting directory sets cursor position back to 1
        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.current_cursor_position(), 1);
        assert_eq!(app.current_dir_path, tmp_dir.path().to_path_buf());
    }

    #[test]
    fn entering_a_new_sub_directory_starts_cursor_position_at_0() {
        let tmp_dir = TempDir::new("tmp_dir").unwrap();
        let nested_dir_path_0 =
            PathBuf::from(format!("{}/nested_dir_0", tmp_dir.path().to_str().unwrap()));
        let nested_dir_path_1 =
            PathBuf::from(format!("{}/nested_dir_1", tmp_dir.path().to_str().unwrap()));
        let _nested_dir_0 = create_dir(&nested_dir_path_0);
        let _nested_dir_1 = create_dir(&nested_dir_path_1);

        let nested_file_path_0 = nested_dir_path_0.join("file_a.txt");
        let nested_file_path_1 = nested_dir_path_0.join("file_b.txt");
        let _nested_file_0 = File::create(&nested_file_path_0).unwrap();
        let _nested_file_1 = File::create(&nested_file_path_1).unwrap();

        let mut app = App::new(tmp_dir.path().to_path_buf());

        assert_eq!(app.current_dir_path, tmp_dir.path().to_path_buf());
        assert_eq!(app.current_cursor_position(), 0);

        // Entering directory sets cursor position to 0, as this is the first time entering
        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.current_cursor_position(), 0);
        assert_eq!(app.current_dir_path, nested_dir_path_0);

        // Can change this directories cursor position
        app.handle_key_event(KeyCode::Down.into());
        assert_eq!(app.current_cursor_position(), 1);

        // Exiting directory sets cursor position back to 0
        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.current_cursor_position(), 0);
        assert_eq!(app.current_dir_path, tmp_dir.path().to_path_buf());

        // Move cursor to other directory
        app.handle_key_event(KeyCode::Down.into());
        assert_eq!(app.current_cursor_position(), 1);

        // Go into this new directory
        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.current_cursor_position(), 0);
        assert_eq!(app.current_dir_path, nested_dir_path_1);

        // Exiting directory again sets cursor position back to 1
        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.current_cursor_position(), 1);
        assert_eq!(app.current_dir_path, tmp_dir.path().to_path_buf());

        // Entering first sub directory and cursor position is 0
        app.handle_key_event(KeyCode::Up.into());
        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.current_cursor_position(), 0);
        assert_eq!(app.current_dir_path, nested_dir_path_0);
    }

    #[test]
    fn default_render_single_column() {
        // TODO: Make this test nicer
        let tmp_dir = TempDir::new("tmp_dir").unwrap();
        let nested_dir_path =
            PathBuf::from(format!("{}/nested_dir", tmp_dir.path().to_str().unwrap()));
        let _nested_dir = create_dir(&nested_dir_path);
        let file_path = tmp_dir.path().join("file.txt");
        let _tmp_file = File::create(&file_path).unwrap();

        let app = App::new(tmp_dir.path().to_path_buf());

        let mut buf = Buffer::empty(Rect::new(0, 0, 81, 5));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ TUI File Explorer ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓",
            &format!("┃{:width$}┃", tmp_dir.path().to_str().unwrap(), width = 79),
            "┃> file.txt                                                                     ┃",
            "┃  nested_dir                                                                   ┃",
            "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
        ]);
        let title_style = Style::new().bold();
        let current_dir_style = Style::new();
        let file_style = Style::new().yellow();
        let dir_style = Style::new().blue();

        let temp_dir_absolute_path_length = tmp_dir.path().to_str().unwrap().len() as u16;
        expected.set_style(Rect::new(31, 0, 19, 1), title_style);
        expected.set_style(
            Rect::new(1, 1, 1 + temp_dir_absolute_path_length, 1),
            current_dir_style,
        );
        expected.set_style(Rect::new(1, 2, 10, 1), file_style);
        expected.set_style(Rect::new(1, 3, 12, 1), dir_style);

        assert_eq!(buf, expected);
    }

    #[test]
    fn default_render_multiple_columns() {
        let tmp_dir = TempDir::new("tmp_dir").unwrap();
        let nested_dir_path =
            PathBuf::from(format!("{}/nested_dir", tmp_dir.path().to_str().unwrap()));
        let _nested_dir = create_dir(&nested_dir_path);
        let file_path_0 = tmp_dir.path().join("file.txt");
        let _tmp_file_0 = File::create(&file_path_0).unwrap();
        let file_path_1 = tmp_dir.path().join("zzz.txt");
        let _tmp_file = File::create(&file_path_1).unwrap();

        let app = App::new(tmp_dir.path().to_path_buf());

        let mut buf = Buffer::empty(Rect::new(0, 0, 81, 5));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ TUI File Explorer ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓",
            &format!("┃{:width$}┃", tmp_dir.path().to_str().unwrap(), width = 79),
            "┃> file.txt          zzz.txt                                                    ┃",
            "┃  nested_dir                                                                   ┃",
            "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
        ]);
        let title_style = Style::new().bold();
        let current_dir_style = Style::new();
        let file_style = Style::new().yellow();
        let dir_style = Style::new().blue();

        let temp_dir_absolute_path_length = tmp_dir.path().to_str().unwrap().len() as u16;
        expected.set_style(Rect::new(31, 0, 19, 1), title_style);
        expected.set_style(
            Rect::new(1, 1, 1 + temp_dir_absolute_path_length, 1),
            current_dir_style,
        );
        expected.set_style(Rect::new(1, 2, 10, 1), file_style);
        expected.set_style(Rect::new(19, 2, 9, 1), file_style);
        expected.set_style(Rect::new(1, 3, 12, 1), dir_style);

        assert_eq!(buf, expected);
    }
}
