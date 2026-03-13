use ratatui::style::Color;

pub fn usage_color(pct: u8) -> Color {
    match pct {
        0..=59 => Color::Green,
        60..=79 => Color::Yellow,
        _ => Color::Red,
    }
}
