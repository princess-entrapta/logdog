#!/bin/sh
set -e

echo "Connect data nodes to cluster and create distributed hypertable..."
psql -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" <<-EOSQL

CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE logs (
    time TIMESTAMP,
    level TEXT,
    source TEXT,
    words TEXT[],
    logdata JSONB
);

CREATE TABLE filters (
    query TEXT,
    name TEXT PRIMARY KEY
);

CREATE TABLE column_filter (
    column_name TEXT,
    filter_name TEXT,
    idx int
);

CREATE TABLE cols (
  query TEXT,
  name TEXT PRIMARY KEY
);

CREATE INDEX idx_logdata ON logs USING GIN (logdata);
CREATE INDEX idx_words ON logs USING GIN (words);

INSERT INTO filters (name, query) VALUES ('logs', 'true');
INSERT INTO column_filter (column_name, filter_name) VALUES ('Data', 'logs');
INSERT INTO cols (name, query) VALUES ('Data', 'logdata');


SELECT create_hypertable('logs', 'time');
SELECT set_chunk_time_interval('logs', INTERVAL '10 minutes');
ALTER TABLE logs SET (timescaledb.compress);
SELECT alter_job((SELECT add_compression_policy('logs', INTERVAL '20 minutes')), schedule_interval => INTERVAL '20 minutes');
SELECT alter_job((SELECT add_retention_policy('logs', INTERVAL '7 days')), schedule_interval => INTERVAL '1 hour');

CREATE MATERIALIZED VIEW logs_sec_count (count, time_bucket) WITH (timescaledb.continuous) AS SELECT Count(*), time_bucket('1s', time) FROM logs GROUP BY time_bucket('1s', time);
SELECT add_continuous_aggregate_policy('logs_sec_count',
  start_offset => '2 minutes',
  end_offset => '1s',
  schedule_interval => INTERVAL '1 minute'); 

CREATE MATERIALIZED VIEW logs_min_count (count, time_bucket) WITH (timescaledb.continuous) AS SELECT Count(*), time_bucket('1 minute', time) FROM logs GROUP BY time_bucket('1 minute', time);
SELECT add_continuous_aggregate_policy('logs_min_count',
  start_offset => '20 minutes',
  end_offset => '1 minute',
  schedule_interval => INTERVAL '10 minute');

EOSQL
