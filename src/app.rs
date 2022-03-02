use crate::input::Prompt;
use crate::storage::Storage;
use crate::ui::explorer::ExplorerState;
use crate::{model::*, storage};
use std::collections::HashMap;

pub struct App {
    pub settings: Settings,

    pub state: State,

    pub storage: Storage,
    pub repository: Repository,
}

#[derive(Default, Debug)]
pub struct Settings {}

#[derive(Default)]
pub struct State {
    pub prompt: Option<Prompt>,
    pub explorer: ExplorerState,
}

#[derive(Default, Debug)]
pub struct Repository {
    pub tags: HashMap<TagId, Tag>,
    pub projects: HashMap<ProjectId, Project>,
    pub tasks: HashMap<TaskId, Task>,
}

impl App {
    pub fn new(settings: Settings, state: State, storage: Storage, repository: Repository) -> Self {
        App {
            settings,
            state,
            storage,
            repository,
        }
    }
}

impl Repository {
    pub fn get_tag_by_name<S>(&self, name: S) -> Option<&Tag>
    where
        S: AsRef<str>,
    {
        let name = name.as_ref();
        self.tags.values().find(|t| t.name.eq(name))
    }

    pub fn get_project_by_name<S>(&self, name: S) -> Option<&Project>
    where
        S: AsRef<str>,
    {
        let name = name.as_ref();
        self.projects.values().find(|p| p.name.eq(name))
    }
}

impl Repository {
    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.insert(tag.id, tag);
    }

    pub fn add_project(&mut self, project: Project) {
        self.projects.insert(project.id, project);
    }
}

pub fn init() -> anyhow::Result<App> {
    let settings = Settings::default();
    let mut state = State::default();
    let storage = storage::init_storage()?;
    let repository = storage::load::load_repository(&storage)?;

    state.explorer.projects.items = repository.projects.keys().cloned().collect();

    Ok(App::new(settings, state, storage, repository))
}
