use std::path::{Path, PathBuf};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    text::{Line, Text},
    widgets::{Paragraph, Widget},
};

pub struct DirectoryView {
    current_dir_contents: Vec<PathBuf>,
    cursor_column_index: usize,
    cursor_row_index: usize,
}

impl DirectoryView {
    pub fn new(
        current_dir_contents: Vec<PathBuf>,
        cursor_column_index: usize,
        cursor_row_index: usize,
    ) -> Self {
        DirectoryView {
            current_dir_contents,
            cursor_column_index,
            cursor_row_index,
        }
    }

    fn get_dir_contents_as_columns(&self, column_height: u16) -> Vec<Vec<PathBuf>> {
        self.current_dir_contents
            .chunks(column_height as usize)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
}

impl Widget for &DirectoryView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dir_contents_columns = self.get_dir_contents_as_columns(area.height);

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
            .split(area);

        for (column_index, (column_area, column_contents)) in
            columns.iter().zip(dir_contents_columns.iter()).enumerate()
        {
            if column_index == self.cursor_column_index {
                Paragraph::new(Text::from(get_formatted_paths(
                    column_contents,
                    Some(self.cursor_row_index),
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

pub fn get_formatted_paths(
    current_dir_contents: &[PathBuf],
    cursor_row_index: Option<usize>,
) -> Vec<Line<'static>> {
    if let Some(cursor_row_index) = cursor_row_index {
        current_dir_contents
            .iter()
            .enumerate()
            .map(|(row_index, entity)| {
                format_path_with_cursor(entity, cursor_row_index == row_index)
            })
            .collect()
    } else {
        current_dir_contents
            .iter()
            .map(|entity| format_path(entity))
            .collect()
    }
}

fn format_path(entity: &Path) -> Line<'static> {
    format_path_with_cursor(entity, false)
}

fn format_path_with_cursor(entity: &Path, with_cursor: bool) -> Line<'static> {
    let prefix = if with_cursor { "> " } else { "  " };

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

#[cfg(test)]
mod tests {
    use std::fs::{File, create_dir};

    use itertools::sorted;
    use ratatui::style::Style;
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn only_file_name_is_shown_not_full_path() {
        assert_eq!(
            format_path_with_cursor(Path::new("/some/nested/file.txt"), false),
            Line::from("  file.txt")
        )
    }

    #[test]
    fn path_without_cursor_has_no_cursor_prefix() {
        assert_eq!(
            format_path_with_cursor(Path::new("file.txt"), false),
            Line::from("  file.txt")
        )
    }

    #[test]
    fn path_with_cursor_has_cursor_prefix() {
        assert_eq!(
            format_path_with_cursor(Path::new("file.txt"), true),
            Line::from("> file.txt")
        )
    }

    #[test]
    fn format_path_passes_with_cursor_as_false() {
        assert_eq!(format_path(Path::new("file.txt")), Line::from("  file.txt"))
    }

    #[test]
    fn list_of_files_formatted_correctly_without_cursor() {
        assert_eq!(
            get_formatted_paths(
                &[PathBuf::from("file_1.txt"), PathBuf::from("file_2.txt")],
                None
            ),
            [Line::from("  file_1.txt"), Line::from("  file_2.txt")]
        )
    }

    #[test]
    fn list_of_files_formatted_correctly_with_cursor() {
        assert_eq!(
            get_formatted_paths(
                &[PathBuf::from("file_1.txt"), PathBuf::from("file_2.txt")],
                Some(1)
            ),
            [Line::from("  file_1.txt"), Line::from("> file_2.txt")]
        )
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

        let directory_contents = sorted(std::fs::read_dir(&tmp_dir).unwrap().filter_map(
            |maybe_dir_entry| {
                let dir_entry = maybe_dir_entry.ok()?;
                Some(dir_entry.path())
            },
        ))
        .collect();

        let directory_view = DirectoryView::new(directory_contents, 0, 0);

        let mut buf = Buffer::empty(Rect::new(0, 0, 81, 3));

        directory_view.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "> file.txt                                                                       ",
            "  nested_dir                                                                     ",
            "                                                                                 ",
        ]);
        let file_style = Style::new().yellow();
        let dir_style = Style::new().blue();

        expected.set_style(Rect::new(0, 0, 10, 1), file_style);
        expected.set_style(Rect::new(0, 1, 12, 1), dir_style);

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
        let directory_contents = sorted(std::fs::read_dir(&tmp_dir).unwrap().filter_map(
            |maybe_dir_entry| {
                let dir_entry = maybe_dir_entry.ok()?;
                Some(dir_entry.path())
            },
        ))
        .collect();

        let directory_view = DirectoryView::new(directory_contents, 0, 0);

        let mut buf = Buffer::empty(Rect::new(0, 0, 81, 2));

        directory_view.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "> file.txt          zzz.txt                                                      ",
            "  nested_dir                                                                     ",
        ]);
        let file_style = Style::new().yellow();
        let dir_style = Style::new().blue();

        expected.set_style(Rect::new(0, 0, 10, 1), file_style);
        expected.set_style(Rect::new(18, 0, 9, 1), file_style);
        expected.set_style(Rect::new(0, 1, 12, 1), dir_style);

        assert_eq!(buf, expected);
    }
}
