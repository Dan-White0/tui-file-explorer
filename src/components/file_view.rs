use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Debug)]
pub struct FileView {
    file_name: String,
    file_contents: Vec<String>,
}

impl FileView {
    pub fn new(file_path: &PathBuf, column_height: usize) -> Self {
        let file_contents = get_formatted_file_contents(file_path, column_height);
        let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();

        FileView {
            file_name,
            file_contents,
        }
    }
}

pub fn get_formatted_file_contents(file_path: &PathBuf, column_height: usize) -> Vec<String> {
    if let Ok(file) = File::open(file_path) {
        let reader = BufReader::new(file);
        let lines = reader.lines().take(column_height).collect();
        if let Ok(lines) = lines {
            lines
        } else {
            vec!["Unable to read contents".to_string()]
        }
    } else {
        vec!["Unable to read file".to_string()]
    }
}

impl Widget for &FileView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let file_name = Line::from(format!(" {} ", self.file_name).bold());
        let file_block = Block::bordered()
            .title(file_name.centered())
            .borders(Borders::LEFT)
            .border_set(border::ROUNDED);

        let formatted_file_contents: Vec<Line> = self
            .file_contents
            .iter()
            .map(|line| {
                // Some characters can be multiple bytes in length
                // This will get the nth character, which is not neccesarily the nth bytes
                let (max_showable_character_index, _) = line
                    .char_indices()
                    .nth(area.width as usize)
                    .unwrap_or((line.len(), 'a'));
                let cropped_line = &line[..max_showable_character_index];
                Line::from(cropped_line)
            })
            .collect();

        Paragraph::new(Text::from(formatted_file_contents))
            .left_aligned()
            .block(file_block)
            .render(area, buf);
    }
}
