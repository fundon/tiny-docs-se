CREATE TABLE IF NOT EXISTS docs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pid INTEGER NOT NULL DEFAULT 0,
    gid TEXT NOT NULL,
    tag INTEGER NOT NULL, -- [1,7]
    locale VARCHAR(32) NOT NULL,
    version VARCHAR(32) NOT NULL,
    content TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS docs_idx_gid ON docs(gid);
CREATE INDEX IF NOT EXISTS docs_idx_pid ON docs(pid);
CREATE INDEX IF NOT EXISTS docs_idx_tag ON docs(tag);
CREATE INDEX IF NOT EXISTS docs_idx_locale ON docs(locale);
CREATE INDEX IF NOT EXISTS docs_idx_version ON docs(version);
