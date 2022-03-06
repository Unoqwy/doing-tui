use super::Storage;
use crate::app::Repository;
use crate::model::*;

macro_rules! fill_map {
    ($repository:ident.$field:ident($local:ident)) => {
        $repository.$field.reserve($local.len());
        for value in $local {
            $repository.$field.insert(value.id, value);
        }
    };
}

macro_rules! parse_concat_row {
    ($row:ident[$idx:expr], $wrap:tt) => {
        $row.get::<usize, Option<String>>($idx)?
            .map(|value| {
                value
                    .split(",")
                    .map(|value| $wrap(value.parse().unwrap()))
                    .collect()
            })
            .unwrap_or_else(|| Vec::new())
    };
}

pub fn load_repository(storage: &Storage) -> anyhow::Result<Repository> {
    let mut repository = Repository::default();

    let mut statement = storage.connection.prepare("SELECT id, name FROM Tag")?;
    let tags: Vec<Tag> = statement
        .query_map([], |row| {
            Ok(Tag {
                id: TagId(row.get(0)?),
                name: row.get(1)?,
            })
        })?
        .map(|tag| tag.unwrap())
        .collect();
    fill_map!(repository.tags(tags));

    let mut statement = storage.connection.prepare(
        "SELECT p.id, p.name, GROUP_CONCAT(DISTINCT Tag.tag_id), GROUP_CONCAT(DISTINCT Task.id) FROM Project p LEFT JOIN DefaultTags Tag ON p.id = Tag.project_id LEFT JOIN Task ON p.id = Task.project_id GROUP BY p.id")?;
    let projects: Vec<Project> = statement
        .query_map([], |row| {
            Ok(Project {
                id: ProjectId(row.get(0)?),
                name: row.get(1)?,
                default_tags: parse_concat_row!(row[2], TagId),
                tasks: parse_concat_row!(row[3], TaskId),
            })
        })?
        .map(|project| project.unwrap())
        .collect();
    fill_map!(repository.projects(projects));

    let mut statement = storage.connection.prepare(
        "SELECT id, project_id, name, GROUP_CONCAT(t.tag_id) FROM Task k LEFT JOIN TaskTags t ON k.id = t.task_id GROUP BY id")?;
    let tasks: Vec<Task> = statement
        .query_map([], |row| {
            Ok(Task {
                id: TaskId(row.get(0)?),
                project_id: ProjectId(row.get(1)?),
                name: row.get(2)?,
                tags: parse_concat_row!(row[3], TagId),
            })
        })?
        .map(|task| task.unwrap())
        .collect();
    fill_map!(repository.tasks(tasks));

    Ok(repository)
}
