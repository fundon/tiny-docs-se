.load libsimple

CREATE VIRTUAL TABLE IF NOT EXISTS d USING fts5(id, pid, gid, tag, locale, version, content, tokenize = 'simple');

INSERT INTO d SELECT * FROM docs;
