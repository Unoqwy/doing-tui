use rusqlite::{params, Connection};

use crate::app::Repository;
use crate::model::*;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub struct Storage {
    connection: Connection,
}

impl Storage {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }
}

impl Storage {
    pub fn create_tag(&self, name: String) -> anyhow::Result<Tag> {
        self.connection
            .execute("INSERT INTO Tag (name) VALUES (?)", params![name])?;
        let id = self.connection.last_insert_rowid();
        Ok(Tag::new(TagId::from(id), name))
    }

    pub fn create_project(&self, name: String) -> anyhow::Result<Project> {
        self.connection
            .execute("INSERT INTO Project (name) VALUES (?)", params![name])?;
        let id = self.connection.last_insert_rowid();
        Ok(Project::new(ProjectId::from(id), name, Vec::new()))
    }

    pub fn create_task(&self, project_id: ProjectId, name: String) -> anyhow::Result<Task> {
        self.connection.execute(
            "INSERT INTO Task (project_id, name) VALUES (?, ?)",
            params![project_id.0, name],
        )?;
        let id = self.connection.last_insert_rowid();
        Ok(Task::new(TaskId::from(id), name, Vec::new()))
    }

    pub fn load_repository(&self) -> anyhow::Result<Repository> {
        let mut repository = Repository::default();

        let mut statement = self.connection.prepare("SELECT id, name FROM Tag")?;
        let tags: Vec<Tag> = statement
            .query_map([], |row| {
                Ok(Tag {
                    id: TagId(row.get(0)?),
                    name: row.get(1)?,
                })
            })?
            .map(|tag| tag.unwrap())
            .collect();
        repository.tags.reserve(tags.len());
        for tag in tags {
            repository.tags.insert(tag.id, tag);
        }

        let mut statement = self.connection.prepare("SELECT id, name, GROUP_CONCAT(t.tag_id) AS tags FROM Project p LEFT JOIN DefaultTags t ON p.id = t.project_id GROUP BY id")?;
        let projects: Vec<Project> = statement
            .query_map([], |row| {
                let tags: Option<String> = row.get(2)?;
                Ok(Project {
                    id: ProjectId(row.get(0)?),
                    name: row.get(1)?,
                    default_tags: tags
                        .map(|tags| {
                            tags.split(",")
                                .map(|tag| TagId(tag.parse().unwrap()))
                                .collect()
                        })
                        .unwrap_or_else(|| Vec::new()),
                })
            })?
            .map(|project| project.unwrap())
            .collect();
        repository.projects.reserve(projects.len());
        for project in projects {
            repository.projects.insert(project.id, project);
        }

        let mut statement = self.connection.prepare("SELECT id, name, GROUP_CONCAT(t.tag_id) AS tags FROM Task k LEFT JOIN TaskTags t ON k.id = t.task_id GROUP BY id")?;
        let tasks: Vec<Task> = statement
            .query_map([], |row| {
                let tags: Option<String> = row.get(2)?;
                Ok(Task {
                    id: TaskId(row.get(0)?),
                    name: row.get(1)?,
                    tags: tags
                        .map(|tags| {
                            tags.split(",")
                                .map(|tag| TagId(tag.parse().unwrap()))
                                .collect()
                        })
                        .unwrap_or_else(|| Vec::new()),
                })
            })?
            .map(|task| task.unwrap())
            .collect();
        repository.tasks.reserve(tasks.len());
        for task in tasks {
            repository.tasks.insert(task.id, task);
        }

        Ok(repository)
    }
}

pub fn init_storage() -> anyhow::Result<Storage> {
    let mut connection = Connection::open("local.db")?;

    embedded::migrations::runner().run(&mut connection).unwrap();

    Ok(Storage::new(connection))
}
