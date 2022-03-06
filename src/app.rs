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
    pub focus: Pane,
    pub prompt: Option<Prompt>,

    pub explorer: ExplorerState,
}

pub enum Pane {
    ProjectExplorer,
    Main,
}

impl Default for Pane {
    fn default() -> Self {
        Self::ProjectExplorer
    }
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

    pub fn sync(&mut self) {
        self.state.explorer.sync(&self.repository);
    }

    pub fn update_focus(&mut self) {
        if self.state.explorer.collapsed {
            self.state.focus = Pane::Main;
        } else {
            self.state.focus = Pane::ProjectExplorer;
        }
    }
}

impl Repository {
    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.insert(tag.id, tag);
    }

    pub fn remove_tag(&mut self, tag_id: &TagId) {
        self.tags.remove(tag_id);
    }

    pub fn add_project(&mut self, project: Project) {
        self.projects.insert(project.id, project);
    }

    pub fn remove_project(&mut self, project_id: &ProjectId) {
        self.projects.remove(project_id);
    }

    pub fn add_task(&mut self, task: Task) {
        if let Some(project) = self.projects.get_mut(&task.project_id) {
            project.tasks.push(task.id);
        }
        self.tasks.insert(task.id, task);
    }

    pub fn remove_task(&mut self, task_id: &TaskId) {
        if let Some(task) = self.tasks.remove(task_id) {
            if let Some(project) = self.projects.get_mut(&task.project_id) {
                let index = project
                    .tasks
                    .iter()
                    .position(|id| id.eq(&task.id))
                    .expect("Tasks were not synced for project");
                project.tasks.remove(index);
            }
        }
    }
}

pub fn init() -> anyhow::Result<App> {
    let settings = Settings::default();
    let state = State::default();
    let storage = storage::init_storage()?;
    let repository = storage::load::load_repository(&storage)?;

    let mut app = App::new(settings, state, storage, repository);
    app.state.explorer.sync(&app.repository);

    Ok(app)
}
