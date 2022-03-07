use crate::input::*;
use crate::model::*;

pub fn new_tag() -> Prompt {
    Prompt::Input(InputPrompt::new("New Tag", 15, true, |app, name| {
        let tag = app.storage.create_tag(name)?;
        app.repository.add_tag(tag);
        app.sync();
        Ok(())
    }))
}

pub fn delete_tag(tag_id: TagId) -> Prompt {
    Prompt::Confirm(ConfirmPrompt::new("deleting selected tag", move |app| {
        app.storage.delete_tag(&tag_id)?;
        app.repository.remove_tag(&tag_id);
        app.sync();
        Ok(())
    }))
}

pub fn new_project() -> Prompt {
    Prompt::Input(InputPrompt::new("New Project", 20, false, |app, name| {
        let project = app.storage.create_project(name)?;
        app.repository.add_project(project);
        app.sync();
        Ok(())
    }))
}

pub fn delete_project(project_id: ProjectId) -> Prompt {
    Prompt::Confirm(ConfirmPrompt::new(
        "deleting selected project",
        move |app| {
            if app.state.explorer.projects.selected > 0 {
                app.state.explorer.projects.selected -= 1;
            }
            app.storage.delete_project(&project_id)?;
            app.repository.remove_project(&project_id);
            app.sync();
            Ok(())
        },
    ))
}

pub fn new_task(project_id: ProjectId) -> Prompt {
    Prompt::Input(InputPrompt::new(
        "New Task",
        150,
        false,
        move |app, name| {
            let task = app.storage.create_task(&project_id, name)?;
            app.repository.add_task(task);
            app.sync();
            Ok(())
        },
    ))
}

pub fn delete_task(task_id: TaskId) -> Prompt {
    Prompt::Confirm(ConfirmPrompt::new("deleting selected task", move |app| {
        app.storage.delete_task(&task_id)?;
        app.repository.remove_task(&task_id);
        app.sync();
        Ok(())
    }))
}
