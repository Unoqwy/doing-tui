CREATE TABLE Tag (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(50) UNIQUE NOT NULL
);

CREATE TABLE Project (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(50) UNIQUE NOT NULL
);
CREATE TABLE DefaultTags (
    project_id INTEGER,
    tag_id INTEGER,
    PRIMARY KEY(project_id, tag_id),
    FOREIGN KEY (project_id) REFERENCES Project(id),
    FOREIGN KEY (tag_id) REFERENCES Tag(id)
);

CREATE TABLE Task (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER,
    name VARCHAR(150) NOT NULL,
    FOREIGN KEY (project_id) REFERENCES Project(id)
);
CREATE TABLE TaskTags (
    task_id INTEGER,
    tag_id INTEGER,
    PRIMARY KEY(task_id, tag_id),
    FOREIGN KEY (task_id) REFERENCES Task(id),
    FOREIGN KEY (tag_id) REFERENCES Tag(id)
);
