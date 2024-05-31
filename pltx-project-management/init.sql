BEGIN;

CREATE TABLE IF NOT EXISTS project (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    position INTEGER NOT NULL,
    archived BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_position ON project (position);


CREATE TABLE IF NOT EXISTS project_label (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    color TEXT NOT NULL,
    position INTEGER NOT NULL,
    archived BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY (project_id)
        REFERENCES project (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_position ON project_label (position);
CREATE INDEX IF NOT EXISTS idx_project_id ON project_label (project_id);


CREATE TABLE IF NOT EXISTS project_list (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    position INTEGER NOT NULL,
    archived BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY (project_id)
        REFERENCES project (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_position ON project_list (position);
CREATE INDEX IF NOT EXISTS idx_project_id ON project_list (project_id);


CREATE TABLE IF NOT EXISTS project_card (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    list_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    important BOOLEAN NOT NULL CHECK (important IN (0, 1)),
    start_date DATETIME,
    due_date DATETIME,
    reminder INTEGER,
    completed BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
    position INTEGER NOT NULL,
    archived BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY (list_id)
        REFERENCES project_list (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (project_id)
        REFERENCES project (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_position ON project_card (position);
CREATE INDEX IF NOT EXISTS idx_project_id ON project_card (project_id);

CREATE TABLE IF NOT EXISTS card_label (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    card_id INTEGER NOT NULL,
    label_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    archived BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY (project_id)
        REFERENCES project (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (card_id)
        REFERENCES project_card (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (label_id)
        REFERENCES project_label (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_position ON card_label (position);
CREATE INDEX IF NOT EXISTS idx_project_id ON card_label (project_id);
CREATE INDEX IF NOT EXISTS idx_card_id ON card_label (card_id);

CREATE TABLE IF NOT EXISTS card_subtask (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    card_id INTEGER NOT NULL,
    value TEXT NOT NULL,
    completed BOOLEAN NOT NULL CHECK (completed IN (0, 1)),
    position INTEGER NOT NULL,
    archived BOOLEAN CHECK (archived IN (0, 1)) DEFAULT 0,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY (project_id)
        REFERENCES project (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE,
    FOREIGN KEY (card_id)
        REFERENCES project_card (id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_position ON card_subtask (position);
CREATE INDEX IF NOT EXISTS idx_project_id ON card_subtask (project_id);
CREATE INDEX IF NOT EXISTS idx_card_id ON card_subtask (card_id);

COMMIT;
