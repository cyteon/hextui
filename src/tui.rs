use crossterm::{
    event::{self, Event, KeyCode}
};
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::path::Path;

use ratatui::widgets::Paragraph;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::layout::{Layout, Constraint, Direction, Alignment};

pub fn run(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();

    let mut file = std::fs::File::open(path)?;
    let size = file.metadata()?.len();

    let mut offset = 0u64;
    let mut bytes_per_row: usize = 16;

    let mut cursor_col: usize = 0;
    let mut cursor_row: usize = 0;

    let mut content_rows: usize = 0;
    let mut viewport_rows: usize = 0;

    loop {
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,

                    KeyCode::PageDown => {
                        offset = (offset + (bytes_per_row * viewport_rows) as u64).min(size.saturating_sub((viewport_rows * bytes_per_row) as u64));
                    }

                    KeyCode::PageUp => {
                        offset = offset.saturating_sub((bytes_per_row * viewport_rows) as u64);
                    }

                    KeyCode::Down => {
                        if cursor_row + 1 < content_rows.min(viewport_rows) {
                            cursor_row += 1;
                        } else {
                            offset = (offset + bytes_per_row as u64).min(size.saturating_sub((viewport_rows * bytes_per_row) as u64));
                        }
                    }

                    KeyCode::Up => {
                        if cursor_row > 0 {
                            cursor_row -= 1;
                        } else {
                            offset = offset.saturating_sub(bytes_per_row as u64);
                        }
                    }

                    KeyCode::Left => {
                        if cursor_col > 0 {
                            cursor_col -= 1;
                        }
                    }

                    KeyCode::Right => {
                        if cursor_col < bytes_per_row - 1 {
                            cursor_col += 1;
                        }
                    }

                    _ => {},
                }
            }
        }

        terminal.draw(|f| {
            let area = f.area();
            viewport_rows = area.height.saturating_sub(2) as usize;
            bytes_per_row = ((area.width as usize).saturating_sub(15) / 4).max(1);

            let mut lines = vec![
                Line::from(format!("{} - {} MB", path.display(), size as f32 * 1e-6))
            ];

            let bytes_to_read = viewport_rows * bytes_per_row;
            let mut buffer = vec![0u8; bytes_to_read];

            file.seek(SeekFrom::Start(offset)).unwrap();
            let bytes_read = file.read(&mut buffer).unwrap();

            content_rows = (bytes_read + bytes_per_row - 1) / bytes_per_row;
            let top_padding = ((viewport_rows).saturating_sub(content_rows)) / 2;

            for _ in 0..top_padding {
                lines.push(Line::from(""));
            }

            for row in 0..content_rows {
                let start = row * bytes_per_row;

                if start >= bytes_read {
                    break;
                }

                let end = (start + bytes_per_row).min(bytes_read);
                let slice = &buffer[start..end];

                let line_offset = offset + start as u64;

                let mut spans = Vec::new();

                spans.push(
                    Span::styled(
                        format!("{:08X}", line_offset),
                        Style::default().fg(Color::Yellow),
                    )
                );

                spans.push(Span::raw("    "));

                for (k, b) in slice.iter().enumerate() {
                    if k == cursor_col && row == cursor_row {
                        spans.push(Span::styled(
                            format!("{:02X}", b),
                            Style::default().bg(Color::White)
                        ));

                        spans.push(Span::raw(" "));
                    } else {
                        spans.push(Span::raw(format!("{:02X} ", b)));
                    }
                }

                spans.push(Span::raw("   "));

                for (k, b) in slice.iter().enumerate() {
                    let c = if b.is_ascii_graphic() || *b == b' ' {
                        *b as char
                    } else {
                        '.'
                    };

                    if k == cursor_col && row == cursor_row {
                        spans.push(Span::styled(
                            c.to_string(),
                            Style::default().fg(Color::Green).bg(Color::White)
                        ));
                    } else {
                        spans.push(Span::styled(
                            c.to_string(),
                            Style::default().fg(Color::Green),
                        ));
                    }
                }

                lines.push(Line::from(spans));
            }

            let paragraph = Paragraph::new(lines)
                .alignment(Alignment::Center);

            f.render_widget(paragraph, area);
        })?;
    }

    ratatui::restore();

    Ok(())
}
