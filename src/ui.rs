use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::Frame;

use crate::app::App;

pub fn draw_frame<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(10)])
        .split(f.size());
    draw_header(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);
}

fn draw_header<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let text = vec![Spans::from(Span::from("Test"))];
    let widget = Paragraph::new(text);
    f.render_widget(widget, area);
}

fn draw_main<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Max(40), Constraint::Percentage(70)])
        .split(area);

    draw_projects_list(f, app, chunks[0]);
    let left_block = Block::default().borders(Borders::ALL).title("Tasks");
    f.render_widget(left_block, chunks[1]);
}

fn draw_projects_list<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let items: Vec<_> = app
        .repository
        .tags
        .values()
        .map(|project| ListItem::new(project.name.clone()))
        .collect();
    let widget = List::new(items).block(Block::default().borders(Borders::ALL).title("Projects"));
    f.render_widget(widget, area)
}
