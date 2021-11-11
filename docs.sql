CREATE TABLE IF NOT EXISTS docs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent INTEGER NOT NULL DEFAULT 0,
    content TEXT NOT NULL,
    kind INTEGER NOT NULL,
    uuid TEXT NOT NULL,
    version VARCHAR(32),
    locale VARCHAR(32)
);

CREATE INDEX IF NOT EXISTS docs_idx_uuid ON docs(uuid);
CREATE INDEX IF NOT EXISTS docs_idx_kind ON docs(kind);
CREATE INDEX IF NOT EXISTS docs_idx_version ON docs(version);
CREATE INDEX IF NOT EXISTS docs_idx_parent ON docs(parent);
