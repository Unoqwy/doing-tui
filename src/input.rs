use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::app::{App, Pane};
use crate::ui::explorer::Explorer;

pub enum Prompt {
    Input(InputPrompt),
    Confirm(ConfirmPrompt),
}

impl Prompt {
    pub fn input(inner: InputPrompt) -> Option<Self> {
        Some(Self::Input(inner))
    }

    pub fn confirm(inner: ConfirmPrompt) -> Option<Self> {
        Some(Self::Confirm(inner))
    }
}

pub struct InputPrompt {
    pub title: String,
    pub limit: usize,
    pub callback: Box<dyn FnOnce(&mut App, String) -> anyhow::Result<()>>,

    pub value: String,
}

impl InputPrompt {
    pub fn new<S, C>(title: S, limit: usize, callback: C) -> Self
    where
        S: Into<String>,
        C: FnOnce(&mut App, String) -> anyhow::Result<()> + 'static,
    {
        Self {
            title: title.into(),
            limit,
            callback: Box::new(callback),
            value: String::new(),
        }
    }
}

pub struct ConfirmPrompt {
    pub action: String,
    pub callback: Box<dyn FnOnce(&mut App) -> anyhow::Result<()>>,
}

impl ConfirmPrompt {
    pub fn new<S, C>(action: S, callback: C) -> Self
    where
        S: Into<String>,
        C: FnOnce(&mut App) -> anyhow::Result<()> + 'static,
    {
        Self {
            action: action.into(),
            callback: Box::new(callback),
        }
    }
}

pub fn handle_event(app: &mut App, event: Event) -> anyhow::Result<bool> {
    if let Event::Key(key) = event {
        if let Some(prompt) = &mut app.state.prompt {
            match prompt {
                Prompt::Input(input) => match key.code {
                    KeyCode::Esc => {
                        app.state.prompt = None;
                    }
                    KeyCode::Char(ch) => {
                        if input.value.len() < input.limit {
                            input.value.push(ch);
                        }
                    }
                    KeyCode::Backspace => {
                        input.value.pop();
                    }
                    KeyCode::Enter => {
                        let prompt = std::mem::replace(&mut app.state.prompt, None).unwrap();
                        if let Prompt::Input(input) = prompt {
                            let callback = input.callback;
                            callback(app, input.value)?;
                        }
                    }
                    _ => {}
                },
                Prompt::Confirm(confirm) => match key.code {
                    KeyCode::Esc => {
                        app.state.prompt = None;
                    }
                    KeyCode::Enter => {
                        let prompt = std::mem::replace(&mut app.state.prompt, None).unwrap();
                        if let Prompt::Confirm(confirm) = prompt {
                            let callback = confirm.callback;
                            callback(app)?;
                        }
                    }
                    _ => {}
                },
            }
        } else {
            match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('?') => {}

                KeyCode::Char(c @ '<') | KeyCode::Char(c @ '>') => {
                    app.state.explorer.collapsed = c == '<';
                    app.update_focus();
                }
                _ => match app.state.focus {
                    Pane::Explorer => handle_explorer_key(key, app)?,
                    Pane::Main => {}
                },
            }
        }
    }
    Ok(false)
}

fn handle_explorer_key(key: KeyEvent, app: &mut App) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.state.explorer.projects.previous();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.state.explorer.projects.next();
        }
        KeyCode::Char('N') => {
            app.state.prompt = Prompt::input(InputPrompt::new("New Project", 20, |app, name| {
                let project = app.storage.create_project(name)?;
                app.repository.add_project(project);
                app.sync();
                Ok(())
            }));
        }
        KeyCode::Char('D') => {
            let project = app.state.explorer.selected_project(app);
            if let Some(project) = project {
                let id = project.id;
                app.state.prompt =
                    Prompt::confirm(ConfirmPrompt::new("deleting a project", move |app| {
                        if app.state.explorer.projects.selected > 0 {
                            app.state.explorer.projects.selected -= 1;
                        }
                        app.storage.delete_project(&id)?;
                        app.repository.remove_project(&id);
                        app.sync();
                        Ok(())
                    }));
            }
        }
        _ => {}
    }
    Ok(())
}
