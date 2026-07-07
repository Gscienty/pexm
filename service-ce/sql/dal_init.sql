CREATE TABLE IF NOT EXISTS users (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    username    TEXT NOT NULL,
    email       TEXT NOT NULL UNIQUE,
    password    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS exam_spaces (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    token           TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    paper_count     INTEGER NOT NULL DEFAULT 0,
    last_updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS exam_space_members (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         INTEGER NOT NULL,
    exam_space_id   INTEGER NOT NULL,
    role            TEXT NOT NULL DEFAULT 'member',
    last_accessed_at DATETIME,
    UNIQUE (user_id, exam_space_id)
);

CREATE TABLE IF NOT EXISTS directories (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    exam_space_id   INTEGER NOT NULL,
    parent_id       INTEGER,
    name            TEXT NOT NULL,
    created_at      DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at      DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_directories_exam_space_id ON directories(exam_space_id);
CREATE INDEX IF NOT EXISTS idx_directories_parent_id ON directories(parent_id);

CREATE TABLE IF NOT EXISTS papers (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    exam_space_id   INTEGER NOT NULL,
    directory_id    INTEGER,
    title           TEXT NOT NULL,
    content         BLOB,
    template_id     INTEGER,
    template_config BLOB,
    created_at      DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at      DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_papers_exam_space_id ON papers(exam_space_id);
CREATE INDEX IF NOT EXISTS idx_papers_directory_id ON papers(directory_id);

-- 试卷模板：存储 Lua 脚本（包含输出配置或 LaTeX 内容的若干函数）
CREATE TABLE IF NOT EXISTS paper_templates (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    content         BLOB,
    preview         BLOB,
    created_at      DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at      DATETIME NOT NULL DEFAULT (datetime('now'))
);
