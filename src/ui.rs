use lazy_static::lazy_static;

use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use tui::Frame;

pub mod explorer;
mod util;

use crate::app::App;
use crate::input::Prompt;
use crate::model::{Project, Tag, Task};

use self::explorer::Explorer;

lazy_static! {
    static ref MARGIN_BLOCK_H: Margin = Margin {
        vertical: 0,
        horizontal: 1,
    };

    // Bindings
    static ref BINDINGS_PROMPT_TAG_SELECT: Vec<(&'static str, &'static str)> = [
    ]
    .into();
}

pub fn draw_frame<B: Backend>(f: &mut Frame<B>, app: &App) {
    if app.state.explorer.collapsed {
        draw_main(f, app, f.size());
    } else {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(40), Constraint::Percentage(70)])
            .split(f.size());
        draw_project_explorer(f, app, chunks[0]);
        draw_main(f, app, chunks[1]);
    }

    draw_prompt(f, app);
}

pub fn draw_prompt_footer<'a, B, Bd>(f: &mut Frame<B>, app: &App, area: Option<Rect>, bindings: Bd)
where
    B: Backend,
    Bd: Into<Vec<(&'a str, &'a str)>>,
{
    if let Some(area) = area {
        let bindings = util::bindings(bindings);
        let stack_length = app.state.prompt_stack.len();
        let stack_length = Paragraph::new(Span::styled(
            format!("({})", stack_length),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::DIM),
        ))
        .alignment(Alignment::Right);
        f.render_widget(bindings, area);
        f.render_widget(stack_length, area);
    }
}

fn draw_prompt<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = f.size();
    if let Some(prompt) = app.prompt() {
        match prompt {
            Prompt::Input(input) => {
                let (area, clear, footer) = util::overlay(area, 3, true);
                f.render_widget(Clear, clear);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(input.title.as_ref());
                let paragraph = Paragraph::new(input.value.as_ref()).block(block);
                f.render_widget(paragraph, area);
                f.set_cursor(area.x + 1 + input.value.len() as u16, area.y + 1);

                let input_len = input.value.len();
                let char_count = format!("{}/{}", input_len, input.limit);
                if area.width as usize > input.title.len() + char_count.len() + 5 {
                    let width = char_count.len() as u16;
                    let mut char_count =
                        Paragraph::new(char_count.as_ref()).alignment(Alignment::Right);
                    if input_len >= input.limit {
                        char_count = char_count.style(Style::default().fg(Color::Red));
                    }
                    let rect = Rect::new(area.x + area.width - 2 - width, area.y, width, 1);
                    f.render_widget(char_count, rect);
                }

                draw_prompt_footer(f, app, footer, [("esc", "cancel"), ("enter", "continue")]);
            }
            Prompt::TagSelect(tag_select) => {
                let (area, clear, footer) = util::overlay(area, 5 + area.height / 3, true);
                f.render_widget(Clear, clear);

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(2),
                        Constraint::Length(1),
                    ])
                    .split(area);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(tag_select.title.as_ref());
                let search = "Search: ";
                let cursor = (
                    chunks[1].x + 1 + (search.len() + tag_select.search.len()) as u16,
                    chunks[1].y,
                );
                let search = Paragraph::new(Spans::from(vec![
                    Span::styled(search, Style::default().add_modifier(Modifier::BOLD)),
                    Span::from(tag_select.search.as_ref()),
                ]));

                let divider = Paragraph::new(format!("├{}┤", "─".repeat(area.width as usize - 2)));

                f.render_widget(block, area);
                f.render_widget(search, chunks[1].inner(&MARGIN_BLOCK_H));
                f.set_cursor(cursor.0, cursor.1);
                f.render_widget(divider, chunks[2]);

                let explorer = &tag_select.explorer;
                explorer::draw_explorer(
                    f,
                    app,
                    chunks[3].inner(&MARGIN_BLOCK_H),
                    explorer,
                    |tag: &Tag, selected| {
                        let mut style = Style::default();
                        if selected {
                            style = style.add_modifier(Modifier::BOLD);
                        }
                        Spans::from(vec![
                            Span::styled("* ", Style::default().add_modifier(Modifier::DIM)),
                            Span::styled(&tag.name, style),
                        ])
                    },
                    |p| p,
                    false,
                );
                if let Some((position, area)) =
                    util::list_position(area, explorer.selected + 1, explorer.items.len())
                {
                    f.render_widget(position, area);
                }

                draw_prompt_footer(
                    f,
                    app,
                    footer,
                    [
                        ("esc", "cancel"),
                        ("enter", "select"),
                        ("ctrl+j", "down"),
                        ("ctrl+k", "up"),
                        ("ctrl+n", "create tag"),
                    ],
                );
            }
            Prompt::Confirm(confirm) => {
                let (area, clear, footer) = util::overlay(area, 3, true);
                f.render_widget(Clear, clear);

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
                f.render_widget(paragraph, area);

                draw_prompt_footer(f, app, footer, [("esc", "cancel"), ("enter", "continue")]);
            }
        }
    }
}

fn draw_project_explorer<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Projects");
    explorer::draw_explorer(
        f,
        app,
        area,
        &app.state.explorer.projects,
        |project: &Project, selected| util::default_list_item(&project.name, selected),
        move |p| p.block(block),
        true,
    );
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
    let explorer = app.state.explorer.tasks();
    let block = Block::default().borders(Borders::ALL).title("Tasks");
    explorer::draw_explorer(
        f,
        app,
        chunks[0],
        explorer,
        |task: &Task, selected| {
            let mut style = Style::default();
            if app.state.explorer.collapsed && selected {
                style = style.add_modifier(Modifier::BOLD);
            }
            Spans::from(vec![
                Span::styled("* ", Style::default().add_modifier(Modifier::DIM)),
                Span::styled(&task.name, style),
            ])
        },
        move |p| p.block(block).wrap(Wrap { trim: true }),
        true,
    );

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
