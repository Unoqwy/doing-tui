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

macro_rules! parse_tags_row {
    ($row:ident[$idx:expr]) => {
        $row.get::<usize, Option<String>>($idx)?
            .map(|tags| {
                tags.split(",")
                    .map(|tag| TagId(tag.parse().unwrap()))
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
        "SELECT id, name, GROUP_CONCAT(t.tag_id) AS tags FROM Project p LEFT JOIN DefaultTags t ON p.id = t.project_id GROUP BY id")?;
    let projects: Vec<Project> = statement
        .query_map([], |row| {
            Ok(Project {
                id: ProjectId(row.get(0)?),
                name: row.get(1)?,
                default_tags: parse_tags_row!(row[2]),
            })
        })?
        .map(|project| project.unwrap())
        .collect();
    fill_map!(repository.projects(projects));

    let mut statement = storage.connection.prepare(
        "SELECT id, name, GROUP_CONCAT(t.tag_id) AS tags FROM Task k LEFT JOIN TaskTags t ON k.id = t.task_id GROUP BY id")?;
    let tasks: Vec<Task> = statement
        .query_map([], |row| {
            Ok(Task {
                id: TaskId(row.get(0)?),
                name: row.get(1)?,
                tags: parse_tags_row!(row[2]),
            })
        })?
        .map(|task| task.unwrap())
        .collect();
    fill_map!(repository.tasks(tasks));

    Ok(repository)
}
