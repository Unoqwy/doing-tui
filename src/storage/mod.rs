use rusqlite::{params, Connection};

use crate::model::*;

pub mod load;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub fn init_storage() -> anyhow::Result<Storage> {
    let mut connection = Connection::open("local.db")?;

    embedded::migrations::runner().run(&mut connection).unwrap();

    Ok(Storage::new(connection))
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
        Ok(Tag {
            id: TagId::from(id),
            name,
        })
    }

    pub fn delete_tag(&self, id: &TagId) -> anyhow::Result<()> {
        self.connection
            .execute("DELETE FROM Tag WHERE id = ?", params![id.0])?;
        Ok(())
    }

    pub fn create_project(&self, name: String) -> anyhow::Result<Project> {
        self.connection
            .execute("INSERT INTO Project (name) VALUES (?)", params![name])?;
        let id = self.connection.last_insert_rowid();
        Ok(Project {
            id: ProjectId::from(id),
            name,
            default_tags: Vec::new(),
            tasks: Vec::new(),
        })
    }

    pub fn delete_project(&self, id: &ProjectId) -> anyhow::Result<()> {
        self.connection
            .execute("DELETE FROM Project WHERE id = ?", params![id.0])?;
        Ok(())
    }

    pub fn create_task(&self, project_id: &ProjectId, name: String) -> anyhow::Result<Task> {
        self.connection.execute(
            "INSERT INTO Task (project_id, name) VALUES (?, ?)",
            params![project_id.0, name],
        )?;
        let id = self.connection.last_insert_rowid();
        Ok(Task {
            id: TaskId::from(id),
            project_id: *project_id,
            name,
            tags: Vec::new(),
        })
    }

    pub fn delete_task(&self, id: &TaskId) -> anyhow::Result<()> {
        self.connection
            .execute("DELETE FROM Task WHERE id = ?", params![id.0])?;
        Ok(())
    }
}
