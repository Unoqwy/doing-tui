use crate::app::App;
use crate::model::{Project, ProjectId, TaskId};

pub trait Explorer {
    fn previous(&mut self);
    fn next(&mut self);
}

#[derive(Debug)]
pub struct ExplorerGroup<T> {
    pub items: Vec<T>,
    pub selected: usize,
}

#[derive(Default, Debug)]
pub struct ExplorerState {
    pub projects: ExplorerGroup<ProjectId>,
    pub tasks: Option<ExplorerGroup<TaskId>>,
    pub tasks_unfocused: bool,
}

impl<T> Default for ExplorerGroup<T> {
    fn default() -> Self {
        Self {
            items: Default::default(),
            selected: Default::default(),
        }
    }
}

impl<T> Explorer for ExplorerGroup<T> {
    fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn next(&mut self) {
        let next = self.selected + 1;
        if next < self.items.len() {
            self.selected = next;
        }
    }
}

impl ExplorerState {
    pub fn projects<'a>(&self, app: &'a App) -> Vec<&'a Project> {
        self.projects
            .items
            .iter()
            .map(|id| {
                app.repository
                    .projects
                    .get(id)
                    .expect("Explorer is out of sync with repository")
            })
            .collect()
    }

    pub fn previous(&mut self) {
        match &mut self.tasks {
            Some(e) if !self.tasks_unfocused => e.previous(),
            _ => self.projects.previous(),
        }
    }

    pub fn next(&mut self) {
        match &mut self.tasks {
            Some(e) if !self.tasks_unfocused => e.next(),
            _ => self.projects.next(),
        }
    }
}
