use crossterm::event::{Event, KeyCode};

use crate::app::App;

pub struct Prompt {
    pub title: String,
    pub limit: usize,
    pub callback: fn(&mut App, String) -> anyhow::Result<()>,

    pub value: String,
}

impl Prompt {
    pub fn new<S>(
        title: S,
        limit: usize,
        callback: fn(&mut App, String) -> anyhow::Result<()>,
    ) -> Self
    where
        S: Into<String>,
    {
        Self {
            title: title.into(),
            limit,
            callback,
            value: String::new(),
        }
    }
}

pub fn handle_event(app: &mut App, event: Event) -> anyhow::Result<bool> {
    if let Event::Key(key) = event {
        if let Some(prompt) = &mut app.state.prompt {
            match key.code {
                KeyCode::Esc => {
                    app.state.prompt = None;
                }
                KeyCode::Char(ch) => {
                    if prompt.value.len() < prompt.limit {
                        prompt.value.push(ch);
                    }
                }
                KeyCode::Backspace => {
                    prompt.value.pop();
                }
                KeyCode::Enter => {
                    let prompt = std::mem::replace(&mut app.state.prompt, None).unwrap();
                    let callback = prompt.callback;
                    callback(app, prompt.value)?;
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    app.state.explorer.previous();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.state.explorer.next();
                }
                KeyCode::Char('N') => {
                    app.state.prompt = Some(Prompt::new("New Project", 20, project_create));
                }
                _ => {}
            }
            match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('?') => {}
                _ => {}
            }
        }
    }
    Ok(false)
}

fn project_create(app: &mut App, name: String) -> anyhow::Result<()> {
    let project = app.storage.create_project(name)?;
    let id = project.id;
    app.repository.add_project(project);
    app.state.explorer.projects.items.push(id);
    Ok(())
}
