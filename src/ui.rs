use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
};

pub fn draw(frame: &mut Frame, _app: &App) {
    // Outer vertical split: top panels (~75%) | log (~20%) | status bar (1 line)
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(75),
            Constraint::Percentage(20),
            Constraint::Length(1),
        ])
        .split(frame.area());

    // Top row: Plans (~25%) | Stories (~75%)
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(vertical[0]);

    frame.render_widget(Block::default().borders(Borders::ALL).title("Plans"), top[0]);
    frame.render_widget(Block::default().borders(Borders::ALL).title("Stories"), top[1]);
    frame.render_widget(Block::default().borders(Borders::ALL).title("Log"), vertical[1]);

    // Status bar: no border
    frame.render_widget(Paragraph::new("[q]uit"), vertical[2]);
}
