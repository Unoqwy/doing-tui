use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use tui::Frame;

pub mod explorer;

use crate::app::App;

pub fn draw_frame<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Max(40), Constraint::Percentage(70)])
        .split(f.size());
    draw_side_pane(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);

    draw_overlays(f, app);
}

fn draw_overlays<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = f.size();
    if let Some(prompt) = &app.state.prompt {
        let width = 70 * area.width / 100;
        let rect = Rect::new((area.width - width) / 2, area.height / 2 - 2, width, 3);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(prompt.title.as_ref());
        let paragraph = Paragraph::new(prompt.value.as_ref()).block(block);
        f.render_widget(Clear, rect);
        f.render_widget(paragraph, rect);
        f.set_cursor(rect.x + 1 + prompt.value.len() as u16, rect.y + 1);

        let input_len = prompt.value.len();
        let char_count = format!("{}/{}", input_len, prompt.limit);
        if rect.width as usize > prompt.title.len() + char_count.len() + 5 {
            let width = char_count.len() as u16;
            let mut char_count = Paragraph::new(char_count.as_ref()).alignment(Alignment::Right);
            if input_len >= prompt.limit {
                char_count = char_count.style(Style::default().fg(Color::Red));
            }
            let rect = Rect::new(rect.x + rect.width - 2 - width, rect.y, width, 1);
            f.render_widget(char_count, rect);
        }
    }
}

fn draw_side_pane<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Projects");
    let selected = app.state.explorer.projects.selected;
    let items: Vec<_> = app
        .state
        .explorer
        .projects(app)
        .into_iter()
        .enumerate()
        .map(|(idx, project)| {
            let style = if idx == selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(project.name.as_ref()).style(style)
        })
        .collect();
    let list = List::new(items).block(block);
    f.render_widget(list, area);

    let total = app.state.explorer.projects.items.len();
    let position = format!("{} of {}", selected + 1, total);
    if area.width as usize > position.len() + 5 {
        let width = position.len() as u16;
        let position = Paragraph::new(position)
            .alignment(Alignment::Right)
            .style(Style::default().add_modifier(Modifier::DIM));
        let rect = Rect::new(
            area.x + area.width - 2 - width,
            area.y + area.height - 1,
            width,
            1,
        );
        f.render_widget(position, rect);
    }
}

fn draw_main<B: Backend>(f: &mut Frame<B>, _app: &App, area: Rect) {
    let left_block = Block::default().borders(Borders::ALL).title("Tasks");
    f.render_widget(left_block, area);
}
