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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskId(pub u32);

#[derive(Clone, Debug)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub tags: Vec<TagId>,
}

impl Tag {
    pub fn new(id: TagId, name: String) -> Self {
        Self { id, name }
    }
}

impl Project {
    pub fn new(id: ProjectId, name: String, default_tags: Vec<TagId>) -> Self {
        Self {
            id,
            name,
            default_tags,
        }
    }
}

impl Task {
    pub fn new(id: TaskId, name: String, tags: Vec<TagId>) -> Self {
        Self { id, name, tags }
    }
}

impl From<&Project> for ProjectId {
    fn from(project: &Project) -> Self {
        project.id
    }
}
