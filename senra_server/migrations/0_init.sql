-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    username        TEXT NOT NULL UNIQUE,
    email           TEXT NOT NULL UNIQUE,
    password        TEXT,
    avatar          BLOB,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create notebooks table
CREATE TABLE IF NOT EXISTS notebooks (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         INTEGER NOT NULL,
    title           TEXT NOT NULL,
    description     TEXT,
    content         JSON NOT NULL,
    preview         BLOB,
    visibility      TEXT NOT NULL DEFAULT 'public',
    version         INTEGER NOT NULL DEFAULT 1,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create notebook_versions table
CREATE TABLE IF NOT EXISTS notebook_versions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    notebook_id     INTEGER NOT NULL,
    user_id         INTEGER NOT NULL,
    version         INTEGER NOT NULL,
    content         JSON NOT NULL,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (notebook_id) REFERENCES notebooks(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- Create notebook_stats table
CREATE TABLE IF NOT EXISTS notebook_stats (
    notebook_id     INTEGER PRIMARY KEY,
    view_count      INTEGER NOT NULL DEFAULT 0,
    like_count      INTEGER NOT NULL DEFAULT 0,
    comment_count   INTEGER NOT NULL DEFAULT 0,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (notebook_id) REFERENCES notebooks(id) ON DELETE CASCADE
);

-- Create notebook_tags table
CREATE TABLE IF NOT EXISTS notebook_tags (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    notebook_id     INTEGER NOT NULL,
    tag             VARCHAR(50) NOT NULL,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (notebook_id, tag),
    FOREIGN KEY (notebook_id) REFERENCES notebooks(id) ON DELETE CASCADE
);

-- Create notebook_likes table
CREATE TABLE IF NOT EXISTS notebook_likes (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    notebook_id     INTEGER NOT NULL,
    user_id         INTEGER NOT NULL,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (notebook_id, user_id),
    FOREIGN KEY (notebook_id) REFERENCES notebooks(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create notebook_comments table
CREATE TABLE IF NOT EXISTS notebook_comments (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    notebook_id     INTEGER NOT NULL,
    user_id         INTEGER NOT NULL,
    content         TEXT NOT NULL,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (notebook_id) REFERENCES notebooks(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create resources table
CREATE TABLE IF NOT EXISTS resources (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    notebook_id     INTEGER NOT NULL,
    name            TEXT NOT NULL,
    resource_type   TEXT NOT NULL,
    data            BLOB NOT NULL,
    metadata        JSON,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (notebook_id) REFERENCES notebooks(id) ON DELETE CASCADE
);

-- Create shaders table
CREATE TABLE IF NOT EXISTS shaders (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    notebook_id     INTEGER NOT NULL,
    name            TEXT NOT NULL,
    shader_type     TEXT NOT NULL,
    code            TEXT NOT NULL,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (notebook_id) REFERENCES notebooks(id) ON DELETE CASCADE
);
