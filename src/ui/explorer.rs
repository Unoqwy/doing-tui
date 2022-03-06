use crate::app::Repository;
use crate::model::{FromId, Project, ProjectId, TaskId};

pub trait Explorer<T> {
    fn previous(&mut self);
    fn next(&mut self);

    fn items<'a, I>(&'a self, repository: &'a Repository) -> Vec<&'a I>
    where
        I: FromId<T>;
    fn selected<'a, I>(&'a self, repository: &'a Repository) -> Option<&'a I>
    where
        I: FromId<T>;
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
    pub tasks: Option<ExplorerGroup<TaskId>>,
}

impl<T> Default for ExplorerGroup<T> {
    fn default() -> Self {
        Self {
            items: Default::default(),
            selected: Default::default(),
        }
    }
}

impl<T> Explorer<T> for ExplorerGroup<T> {
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

    fn items<'a, I>(&'a self, repository: &'a Repository) -> Vec<&'a I>
    where
        I: FromId<T>,
    {
        self.items
            .iter()
            .map(|id| I::from_id(id, repository))
            .collect()
    }

    fn selected<'a, I>(&'a self, repository: &'a Repository) -> Option<&'a I>
    where
        I: FromId<T>,
    {
        self.items
            .get(self.selected)
            .map(|id| I::from_id(id, repository))
    }
}

impl ExplorerState {
    pub fn sync(&mut self, repository: &Repository) {
        Self::sync_explorer(
            &mut self.projects,
            repository.projects.values().collect(),
            |item| item.name.clone(),
        );
        self.project_changed(repository);
    }

    pub fn project_changed(&mut self, repository: &Repository) {
        if let Some(project) = self.projects.selected::<Project>(repository) {
            let mut tasks = ExplorerGroup::default();
            Self::sync_explorer(
                &mut tasks,
                repository
                    .tasks
                    .values()
                    .filter(|task| project.tasks.contains(&task.id))
                    .collect(),
                |item| item.name.clone(),
            );
            self.tasks = Some(tasks);
        } else {
            self.tasks = None;
        }
    }

    fn sync_explorer<'m, Id, M, F, K>(
        explorer: &mut ExplorerGroup<Id>,
        mut items: Vec<&'m M>,
        sort_key: F,
    ) where
        Id: From<&'m M> + Clone + PartialEq,
        F: FnMut(&&M) -> K,
        K: Ord,
    {
        let selected_id = explorer.items.get(explorer.selected).cloned();
        items.sort_by_cached_key(sort_key);
        let items: Vec<Id> = items.into_iter().map(|project| project.into()).collect();
        explorer.selected = selected_id
            .and_then(|id| items.iter().position(|item| item.eq(&id)))
            .unwrap_or(0);
        explorer.items = items;
    }

    pub fn tasks(&self) -> &ExplorerGroup<TaskId> {
        self.tasks.as_ref().expect("Tasks explorer is not synced")
    }
}
