use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use tui::Frame;

pub mod explorer;

use crate::app::App;
use crate::input::Prompt;

pub fn draw_frame<B: Backend>(f: &mut Frame<B>, app: &App) {
    if app.state.explorer.collapsed {
        draw_main(f, app, f.size());
    } else {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(40), Constraint::Percentage(70)])
            .split(f.size());
        draw_explorer(f, app, chunks[0]);
        draw_main(f, app, chunks[1]);
    }

    draw_overlays(f, app);
}

fn draw_overlays<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = f.size();
    if let Some(prompt) = &app.state.prompt {
        let width = 70 * area.width / 100;
        let rect = Rect::new((area.width - width) / 2, area.height / 2 - 2, width, 3);

        match prompt {
            Prompt::Input(input) => {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(input.title.as_ref());
                let paragraph = Paragraph::new(input.value.as_ref()).block(block);
                f.render_widget(Clear, rect);
                f.render_widget(paragraph, rect);
                f.set_cursor(rect.x + 1 + input.value.len() as u16, rect.y + 1);

                let input_len = input.value.len();
                let char_count = format!("{}/{}", input_len, input.limit);
                if rect.width as usize > input.title.len() + char_count.len() + 5 {
                    let width = char_count.len() as u16;
                    let mut char_count =
                        Paragraph::new(char_count.as_ref()).alignment(Alignment::Right);
                    if input_len >= input.limit {
                        char_count = char_count.style(Style::default().fg(Color::Red));
                    }
                    let rect = Rect::new(rect.x + rect.width - 2 - width, rect.y, width, 1);
                    f.render_widget(char_count, rect);
                }
            }
            Prompt::Confirm(confirm) => {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title("Confirmation required");
                let paragraph = Paragraph::new(Spans::from(vec![
                    Span::from("Proceed with "),
                    Span::styled(
                        &confirm.action,
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::from("?"),
                ]))
                .block(block);
                f.render_widget(Clear, rect);
                f.render_widget(paragraph, rect);
            }
        }
    }
}

fn draw_explorer<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
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

fn draw_main<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    if let Some(project) = app.state.explorer.selected_project(&app) {
        let header = vec![Spans::from(vec![
            Span::from(">> "),
            Span::styled(&project.name, Style::default().add_modifier(Modifier::BOLD)),
        ])];
        let header = Paragraph::new(header);
        f.render_widget(header, area);

        let area = Rect::new(area.x, area.y + 1, area.width, area.height - 1);
        let left_block = Block::default().borders(Borders::ALL);
        f.render_widget(left_block, area);
    }
}
