use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use tui::Frame;

pub mod explorer;

use crate::app::App;
use crate::input::Prompt;
use crate::model::{Project, Task};

use self::explorer::Explorer;

fn list_position<'a>(area: Rect, position: usize, total: usize) -> Option<(Paragraph<'a>, Rect)> {
    let position = format!("{} of {}", usize::min(position, total), total);
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
        Some((position, rect))
    } else {
        None
    }
}

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
        .projects
        .items::<Project>(&app.repository)
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
    if let Some((p, rect)) = list_position(area, selected + 1, total) {
        f.render_widget(p, rect);
    }
}

fn draw_main<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    if let Some(project) = app
        .state
        .explorer
        .projects
        .selected::<Project>(&app.repository)
    {
        // Breadcrumb
        let mut breadcrumb = vec![
            Span::styled(">> ", Style::default().add_modifier(Modifier::DIM)),
            Span::styled(&project.name, Style::default().add_modifier(Modifier::BOLD)),
        ];
        if app.state.explorer.collapsed {
            if let Some(task) = app.state.explorer.tasks().selected::<Task>(&app.repository) {
                breadcrumb.push(Span::styled(
                    " > ",
                    Style::default().add_modifier(Modifier::DIM),
                ));
                breadcrumb.push(Span::from(task.name.as_ref()));
            }
        }
        let breadcrumb = vec![Spans::from(breadcrumb)];
        let breadcrumb = Paragraph::new(breadcrumb);
        f.render_widget(breadcrumb, area);

        // Project pane
        let area = Rect::new(area.x, area.y + 1, area.width, area.height - 1);
        draw_project_pane(f, app, area, project);
    } else {
        let area = Rect::new(area.x, area.y + (area.height / 2) - 1, area.width, 2);
        let lines = vec![
            Spans::from("No projects created yet."),
            Spans::from("Press N to create one."),
        ];
        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

fn draw_project_pane<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect, project: &Project) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Task explorer
    let block = Block::default().borders(Borders::ALL).title("Tasks");
    let explorer = app.state.explorer.tasks();
    let tasks: Vec<_> = explorer
        .items::<Task>(&app.repository)
        .into_iter()
        .enumerate()
        .map(|(idx, task)| {
            let style = if app.state.explorer.collapsed && idx == explorer.selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Spans::from(vec![
                Span::styled("* ", Style::default().add_modifier(Modifier::DIM)),
                Span::styled(&task.name, style),
            ])
        })
        .collect();
    let tasks = Paragraph::new(tasks).block(block).wrap(Wrap { trim: true });
    f.render_widget(tasks, chunks[0]);
    if let Some((p, rect)) = list_position(chunks[0], explorer.selected + 1, explorer.items.len()) {
        f.render_widget(p, rect);
    }

    // Task pane
    if let Some(task) = explorer.selected::<Task>(&app.repository) {
        draw_task_pane(f, app, chunks[1], project, task);
    } else {
        let area = chunks[1];
        let area = Rect::new(area.x, area.y + (area.height / 2) - 1, area.width, 2);
        let lines = vec![
            Spans::from("No tasks added to this project yet."),
            Spans::from("Press N to create one."),
        ];
        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

fn draw_task_pane<B: Backend>(
    f: &mut Frame<B>,
    app: &App,
    area: Rect,
    _project: &Project,
    task: &Task,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    // About
    let block = Block::default().borders(Borders::ALL).title("About");
    let about = vec![
        Spans::from(vec![
            Span::from("Added on: "),
            Span::styled(
                "19th Jan 2038",
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Spans::from(vec![
            Span::from("Time spent: "),
            Span::styled("XXhXX", Style::default().add_modifier(Modifier::BOLD)),
        ]),
    ];
    let about = Paragraph::new(about).block(block);
    f.render_widget(about, chunks[0]);

    // Tags
    let block = Block::default().borders(Borders::ALL).title("Tags");
    let tags = Paragraph::new("None").block(block);
    f.render_widget(tags, chunks[1]);

    // Time entries
    let block = Block::default().borders(Borders::ALL).title("Time");
    f.render_widget(block, chunks[2]);
}
