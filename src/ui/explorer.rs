use crate::app::{App, Repository};
use crate::model::{Project, ProjectId};

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
    pub collapsed: bool,
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
    pub fn sync(&mut self, repository: &Repository) {
        let selected_id = self.projects.items.get(self.projects.selected).cloned();
        let mut items: Vec<&Project> = repository.projects.values().collect();
        items.sort_by_cached_key(|item| item.name.clone());
        let items: Vec<ProjectId> = items.into_iter().map(|project| project.id).collect();
        self.projects.selected = selected_id
            .and_then(|id| items.iter().position(|item| item.eq(&id)))
            .unwrap_or(0);
        self.projects.items = items;
    }

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

    pub fn selected_project<'a>(&self, app: &'a App) -> Option<&'a Project> {
        self.projects.items.get(self.projects.selected).map(|id| {
            app.repository
                .projects
                .get(id)
                .expect("Explorer is out of sync with repository")
        })
    }
}
