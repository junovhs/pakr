use crate::types::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use super::panels;

pub fn render(frame: &mut Frame, state: &mut AppState) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let mut iter = chunks.iter().copied();
    if let (Some(left), Some(right)) = (iter.next(), iter.next()) {
        panels::left::render(frame, left, state);
        panels::right::render(frame, right, state);
    }
}
