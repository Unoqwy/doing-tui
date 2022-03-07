use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Pane, Repository};
use crate::model::TagId;
use crate::prompts;
use crate::ui::explorer::{Explorer, ExplorerGroup};

pub enum Prompt {
    Input(InputPrompt),
    TagSelect(TagSelectPrompt),
    Confirm(ConfirmPrompt),
}

impl Prompt {
    pub fn suggest(self, suggest: String) -> Self {
        match self {
            Self::Input(input) => Self::Input(input.suggest(suggest)),
            _ => unimplemented!("Suggest on prompt is only a shorthand for input prompts"),
        }
    }

    pub fn awake(&mut self, repository: &Repository) {
        match self {
            Self::TagSelect(tag_select) => tag_select.update(repository),
            _ => {}
        }
    }
}

pub struct InputPrompt {
    pub title: String,
    pub callback: Box<dyn FnOnce(&mut App, String) -> anyhow::Result<()>>,

    pub limit: usize,
    pub alphanumeric: bool,
    pub value: String,
}

impl InputPrompt {
    pub fn new<S, C>(title: S, limit: usize, only_ascii: bool, callback: C) -> Self
    where
        S: Into<String>,
        C: FnOnce(&mut App, String) -> anyhow::Result<()> + 'static,
    {
        Self {
            title: title.into(),
            callback: Box::new(callback),
            limit,
            alphanumeric: only_ascii,
            value: String::new(),
        }
    }

    pub fn suggest(mut self, suggest: String) -> Self {
        self.value = suggest;
        self
    }
}

pub struct TagSelectPrompt {
    pub title: String,
    pub callback: Box<dyn FnOnce(&mut App, TagId) -> anyhow::Result<()>>,

    pub search: String,
    pub explorer: ExplorerGroup<TagId>,
}

impl TagSelectPrompt {
    pub fn new<S, C>(title: S, callback: C) -> Self
    where
        S: Into<String>,
        C: FnOnce(&mut App, TagId) -> anyhow::Result<()> + 'static,
    {
        Self {
            title: title.into(),
            callback: Box::new(callback),
            search: String::new(),
            explorer: ExplorerGroup::default(),
        }
    }

    fn update(&mut self, repository: &Repository) {
        let prefix = &self.search;
        let no_filter = prefix.is_empty();
        let items = repository
            .tags
            .values()
            .filter(|tag| {
                no_filter || (tag.name.len() >= prefix.len() && tag.name.starts_with(prefix))
            })
            .collect();
        self.explorer.sync_and_sort(items, |item| item.name.clone());
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
        if let Some(prompt) = app.state.prompt_stack.last_mut() {
            match prompt {
                Prompt::Input(input) => match key.code {
                    KeyCode::Esc => {
                        app.close_prompt();
                    }
                    KeyCode::Enter => {
                        if !input.value.is_empty() {
                            if let Some(Prompt::Input(input)) = app.close_prompt() {
                                let callback = input.callback;
                                callback(app, input.value)?;
                                app.awake_prompt();
                            }
                        }
                    }

                    KeyCode::Char(ch) => {
                        if key.modifiers.difference(KeyModifiers::SHIFT).is_empty()
                            && (!input.alphanumeric || ch.is_alphanumeric())
                        {
                            if input.value.len() < input.limit {
                                input.value.push(ch);
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        input.value.pop();
                    }
                    _ => {}
                },
                Prompt::TagSelect(tag_select) => match key.code {
                    KeyCode::Esc => {
                        app.close_prompt();
                    }
                    KeyCode::Enter => {
                        if let Some(selected) = tag_select.explorer.selected_raw().cloned() {
                            if let Some(Prompt::TagSelect(tag_select)) = app.close_prompt() {
                                let callback = tag_select.callback;
                                callback(app, selected)?;
                                app.awake_prompt();
                            }
                        }
                    }

                    KeyCode::Up => {
                        tag_select.explorer.previous();
                    }
                    KeyCode::Down => {
                        tag_select.explorer.next();
                    }

                    KeyCode::Char(ch) => {
                        if key.modifiers.difference(KeyModifiers::SHIFT).is_empty() {
                            if ch.is_alphanumeric() {
                                tag_select.search.push(ch);
                                tag_select.update(&app.repository);
                            }
                        } else if key.modifiers.contains(KeyModifiers::CONTROL) {
                            match ch {
                                'k' => {
                                    tag_select.explorer.previous();
                                }
                                'j' => {
                                    tag_select.explorer.next();
                                }

                                'n' => {
                                    let suggest = tag_select.search.clone();
                                    app.show_prompt(prompts::new_tag().suggest(suggest));
                                }
                                'd' => {
                                    if let Some(tag_id) =
                                        tag_select.explorer.selected_raw().cloned()
                                    {
                                        app.show_prompt(prompts::delete_tag(tag_id));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        tag_select.search.pop();
                        tag_select.update(&app.repository);
                    }
                    _ => {}
                },
                Prompt::Confirm(_) => match key.code {
                    KeyCode::Esc => {
                        app.close_prompt();
                    }
                    KeyCode::Enter => {
                        if let Some(Prompt::Confirm(confirm)) = app.close_prompt() {
                            let callback = confirm.callback;
                            callback(app)?;
                            app.awake_prompt();
                        }
                    }
                    _ => {}
                },
            }
        } else {
            match key.code {
                KeyCode::Esc => {
                    if matches!(app.state.focus, Pane::Main) {
                        app.state.explorer.collapsed = false;
                        app.update_focus();
                    } else {
                        return Ok(true);
                    }
                }
                KeyCode::Enter => {
                    if matches!(app.state.focus, Pane::ProjectExplorer) {
                        app.state.explorer.collapsed = true;
                        app.update_focus();
                    }
                }

                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('?') => {}
                KeyCode::Char(c @ '<') | KeyCode::Char(c @ '>') => {
                    app.state.explorer.collapsed = c == '<';
                    app.update_focus();
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
            app.show_prompt(prompts::new_project());
        }
        KeyCode::Char('D') => {
            if let Some(project_id) = app.state.explorer.projects.selected_raw().cloned() {
                app.show_prompt(prompts::delete_project(project_id));
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_main_key(key: KeyEvent, app: &mut App) -> anyhow::Result<()> {
    if let Some(project_id) = app.state.explorer.projects.selected_raw() {
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
                app.show_prompt(prompts::new_task(project_id));
            }
            KeyCode::Char('D') => {
                if let Some(id) = tasks.selected_raw().cloned() {
                    app.show_prompt(prompts::delete_task(id));
                }
            }

            KeyCode::Char('t') => {
                if let Some(task_id) = tasks.selected_raw().cloned() {
                    let mut prompt = TagSelectPrompt::new("Add tag to task", move |app, tag_id| {
                        // todo
                        Ok(())
                    });
                    prompt.update(&app.repository);
                    app.show_prompt(Prompt::TagSelect(prompt));
                }
            }
            _ => {}
        }
    }
    Ok(())
}
