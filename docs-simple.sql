.load libsimple

CREATE VIRTUAL TABLE IF NOT EXISTS d USING fts5(id, parent, content, kind, uuid, version, locale, tokenize = 'simple');

INSERT INTO d SELECT * FROM docs;
