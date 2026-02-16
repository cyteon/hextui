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
    let mut bytes_per_row: usize = 16; // updated each frame

    loop {
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,

                    KeyCode::Down => {
                        offset = (offset + bytes_per_row as u64).min(size);
                    }

                    KeyCode::Up => {
                        offset = offset.saturating_sub(bytes_per_row as u64);
                    }

                    _ => {},
                }
            }
        }

        terminal.draw(|f| {
            let area = f.area();
            let rows = area.height.saturating_sub(2) as usize;

            bytes_per_row = ((area.width as usize).saturating_sub(15) / 4).max(1);

            let mut lines = vec![
                Line::from(format!("{} - {} MB", path.display(), size as f32 * 1e-6))
            ];

            let bytes_to_read = rows * bytes_per_row;
            let mut buffer = vec![0u8; bytes_to_read];

            file.seek(SeekFrom::Start(offset)).unwrap();
            let bytes_read = file.read(&mut buffer).unwrap();

            let content_width: u16 = (8 + 4 + 3 * bytes_per_row + 3 + bytes_per_row) as u16;

            let horizontal = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(content_width),
                    Constraint::Min(0),
                ])
                .split(area);

            let vertical = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(((bytes_read + bytes_per_row - 1) / bytes_per_row) as u16),
                    Constraint::Min(0),
                ])
                .split(horizontal[1]);

            let centered_area = vertical[1];

            for row in 0..rows {
                let start = row * bytes_per_row;

                if start >= bytes_read {
                    break;
                }

                let end = (start + bytes_per_row).min(bytes_read);
                let slice = &buffer[start..end];

                let offset = offset + start as u64;

                let mut spans = Vec::new();

                spans.push(
                    Span::styled(
                        format!("{:08X}", offset),
                        Style::default().fg(Color::Yellow),
                    )
                );

                spans.push(Span::raw("    "));

                for b in slice {
                    spans.push(Span::raw(format!("{:02X} ", b)));
                }

                spans.push(Span::raw("   "));
                
                for b in slice {
                    let c = if b.is_ascii_graphic() || *b == b' ' {
                        *b as char
                    } else {
                        '.'
                    };

                    spans.push(Span::styled(
                        c.to_string(),
                        Style::default().fg(Color::Green),
                    ));
                }

                lines.push(Line::from(spans));
            }
            
            let paragraph = Paragraph::new(lines);

            f.render_widget(paragraph, centered_area);
        })?;
    }

    ratatui::restore();

    Ok(())
}