use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::app::{App, Pane};
use crate::model::{ProjectId, TaskId};
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
                        if !input.value.is_empty() {
                            let prompt = std::mem::replace(&mut app.state.prompt, None).unwrap();
                            if let Prompt::Input(input) = prompt {
                                let callback = input.callback;
                                callback(app, input.value)?;
                            }
                        }
                    }
                    _ => {}
                },
                Prompt::Confirm(_) => match key.code {
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
                KeyCode::Enter => {
                    if matches!(app.state.focus, Pane::ProjectExplorer) {
                        app.state.explorer.collapsed = true;
                        app.update_focus();
                    }
                }
                KeyCode::Esc => {
                    if matches!(app.state.focus, Pane::Main) {
                        app.state.explorer.collapsed = false;
                        app.update_focus();
                    }
                }
                _ => match app.state.focus {
                    Pane::ProjectExplorer => handle_project_explorer_key(key, app)?,
                    Pane::Main => {
                        handle_main_key(key, app)?;
                    }
                },
            }
        }
    }
    Ok(false)
}

fn handle_project_explorer_key(key: KeyEvent, app: &mut App) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.state.explorer.projects.previous();
            app.state.explorer.project_changed(&app.repository);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.state.explorer.projects.next();
            app.state.explorer.project_changed(&app.repository);
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
            if let Some(project_id) = app
                .state
                .explorer
                .projects
                .selected::<ProjectId>(&app.repository)
                .cloned()
            {
                app.state.prompt = Prompt::confirm(ConfirmPrompt::new(
                    "deleting selected project",
                    move |app| {
                        if app.state.explorer.projects.selected > 0 {
                            app.state.explorer.projects.selected -= 1;
                        }
                        app.storage.delete_project(&project_id)?;
                        app.repository.remove_project(&project_id);
                        app.sync();
                        Ok(())
                    },
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_main_key(key: KeyEvent, app: &mut App) -> anyhow::Result<()> {
    if let Some(project_id) = app
        .state
        .explorer
        .projects
        .selected::<ProjectId>(&app.repository)
    {
        let tasks = app
            .state
            .explorer
            .tasks
            .as_mut()
            .expect("Explorer tasks not in sync");
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                tasks.previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                tasks.next();
            }
            KeyCode::Char('N') => {
                let project_id = *project_id;
                app.state.prompt =
                    Prompt::input(InputPrompt::new("New Task", 150, move |app, name| {
                        let task = app.storage.create_task(&project_id, name)?;
                        app.repository.add_task(task);
                        app.sync();
                        Ok(())
                    }));
            }
            KeyCode::Char('D') => {
                let id = tasks.selected::<TaskId>(&app.repository).cloned();
                if let Some(id) = id {
                    app.state.prompt =
                        Prompt::confirm(ConfirmPrompt::new("deleting selected task", move |app| {
                            app.storage.delete_task(&id)?;
                            app.repository.remove_task(&id);
                            app.sync();
                            Ok(())
                        }));
                }
            }
            _ => {}
        }
    }
    Ok(())
}
