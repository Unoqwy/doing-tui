use tui::backend::Backend;
use tui::layout::Rect;
use tui::text::Spans;
use tui::widgets::Paragraph;
use tui::Frame;

use crate::app::{App, Repository};
use crate::model::{FromId, Project, ProjectId, TaskId};

use super::util;

pub trait Explorer<T> {
    fn previous(&mut self);
    fn next(&mut self);

    fn items<'a, I>(&'a self, repository: &'a Repository) -> Vec<&'a I>
    where
        I: FromId<T>;

    fn selected_raw<'a>(&'a self) -> Option<&'a T>;
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

    fn selected_raw<'a>(&'a self) -> Option<&'a T> {
        self.items.get(self.selected)
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

impl<Id> ExplorerGroup<Id> {
    pub fn sync_and_sort<'m, M, F, K>(&mut self, mut items: Vec<&'m M>, sort_key: F)
    where
        Id: From<&'m M> + Clone + PartialEq,
        F: FnMut(&&M) -> K,
        K: Ord,
    {
        let selected_id = self.items.get(self.selected).cloned();
        items.sort_by_cached_key(sort_key);
        let items: Vec<Id> = items.into_iter().map(|project| project.into()).collect();
        self.selected = selected_id
            .and_then(|id| items.iter().position(|item| item.eq(&id)))
            .unwrap_or(0);
        self.items = items;
    }
}

impl ExplorerState {
    pub fn sync(&mut self, repository: &Repository) {
        self.projects
            .sync_and_sort(repository.projects.values().collect(), |item| {
                item.name.clone()
            });
        self.project_changed(repository);
    }

    pub fn project_changed(&mut self, repository: &Repository) {
        if let Some(project) = self.projects.selected::<Project>(repository) {
            let mut tasks = ExplorerGroup::default();
            tasks.sync_and_sort(
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

    pub fn tasks(&self) -> &ExplorerGroup<TaskId> {
        self.tasks.as_ref().expect("Tasks explorer is not synced")
    }
}

pub fn draw_explorer<B, E, T, Tf, Tp>(
    f: &mut Frame<B>,
    app: &App,
    area: Rect,
    explorer: &ExplorerGroup<E>,
    item_to_spans: Tf,
    transform_paragraph: Tp,
    with_position: bool,
) where
    B: Backend,
    T: FromId<E>,
    Tf: Fn(&T, bool) -> Spans,
    Tp: FnOnce(Paragraph) -> Paragraph,
{
    let items: Vec<_> = explorer
        .items::<T>(&app.repository)
        .into_iter()
        .enumerate()
        .map(|(idx, item)| item_to_spans(item, idx == explorer.selected))
        .collect();
    let items = transform_paragraph(Paragraph::new(items));
    f.render_widget(items, area);

    if with_position {
        if let Some((position, area)) =
            util::list_position(area, explorer.selected + 1, explorer.items.len())
        {
            f.render_widget(position, area);
        }
    }
}
