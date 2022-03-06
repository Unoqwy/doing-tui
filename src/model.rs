use crate::app::Repository;

macro_rules! impl_id {
    ($($struct:ty),*) => {
        $( impl From<i64> for $struct {
            fn from(value: i64) -> Self {
                Self(value as u32)
            }
        })*
    };
}

impl_id!(TagId, ProjectId, TaskId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TagId(pub u32);

#[derive(Clone, Debug)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProjectId(pub u32);

#[derive(Clone, Debug)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub default_tags: Vec<TagId>,
    pub tasks: Vec<TaskId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskId(pub u32);

#[derive(Clone, Debug)]
pub struct Task {
    pub id: TaskId,
    pub project_id: ProjectId,
    pub name: String,
    pub tags: Vec<TagId>,
}

impl From<&Project> for ProjectId {
    fn from(project: &Project) -> Self {
        project.id
    }
}

impl From<&Task> for TaskId {
    fn from(task: &Task) -> Self {
        task.id
    }
}

pub trait FromId<Id> {
    fn from_id<'a>(id: &'a Id, repository: &'a Repository) -> &'a Self;
}

impl FromId<ProjectId> for Project {
    fn from_id<'a>(id: &ProjectId, repository: &'a Repository) -> &'a Self {
        repository
            .projects
            .get(id)
            .expect("Repository is out of sync (projects)")
    }
}

impl FromId<ProjectId> for ProjectId {
    fn from_id<'a>(id: &'a ProjectId, _: &'a Repository) -> &'a Self {
        id
    }
}

impl FromId<TaskId> for Task {
    fn from_id<'a>(id: &TaskId, repository: &'a Repository) -> &'a Self {
        repository
            .tasks
            .get(id)
            .expect("Repository is out of sync (tasks)")
    }
}

impl FromId<TaskId> for TaskId {
    fn from_id<'a>(id: &'a TaskId, _: &'a Repository) -> &'a Self {
        id
    }
}
