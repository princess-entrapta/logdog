CREATE TABLE logs (
    time TIMESTAMP,
    level TEXT,
    source TEXT,
    words TEXT [],
    logdata JSONB
);

CREATE TABLE filters (query TEXT, name TEXT PRIMARY KEY);

CREATE TABLE column_filter (
    column_name TEXT,
    filter_name TEXT,
    idx int
);

CREATE TABLE cols (
    query TEXT,
    name TEXT PRIMARY KEY,
    metric_agg TEXT
);

CREATE INDEX idx_logdata ON logs USING GIN (logdata);

CREATE INDEX idx_words ON logs USING GIN (words);

INSERT INTO
    filters (name, query)
VALUES
    ('logs', 'true');

INSERT INTO
    column_filter (column_name, filter_name)
VALUES
    ('Data', 'logs');

INSERT INTO
    cols (name, query, metric_agg)
VALUES
    ('Data', 'logdata', '');