use std::cmp::max;

use bigdecimal::ToPrimitive;
use chrono::{NaiveDateTime, Utc};
use sqlx::{types::BigDecimal, PgPool, Row};

#[derive(Clone)]
pub struct Repository {
    pub pool: PgPool,
}

impl Repository {
    pub async fn connect(pg_url: &str) -> Self {
        Self {
            pool: PgPool::connect(pg_url).await.unwrap(),
        }
    }
    pub async fn list_filters(
        &self,
    ) -> Result<Vec<(String, Vec<String>, Vec<String>)>, sqlx::error::Error> {
        // TODO: make a type
        let ret: Vec<(String, Vec<String>, Vec<String>)> = sqlx::query(
            "
        SELECT filters.name, array_agg(cols.metric_agg ORDER BY idx), array_agg(cols.name ORDER BY idx) 
            FROM filters 
                JOIN column_filter ON filters.name = column_filter.filter_name 
                JOIN cols ON cols.name = column_filter.column_name 
            GROUP BY filters.name;",
        )
        .fetch_all(&self.pool).await?.into_iter().map(|row| (
            row.get::<String, _>(0),
            row.get::<Vec<String>, _>(1),
            row.get::<Vec<String>, _>(2),
        )).collect::<Vec<(String, Vec<String>, Vec<String>)>>();
        Ok(ret)
    }

    pub async fn get_col_names(&self) -> Result<Vec<String>, sqlx::error::Error> {
        let rows = sqlx::query("SELECT name FROM cols")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| r.get::<String, _>(0).into())
            .collect::<Vec<String>>())
    }

    pub async fn get_metric_query_agg(
        &self,
        metric_name: String,
    ) -> Result<(String, String), sqlx::error::Error> {
        let row = sqlx::query("SELECT query, metric_agg FROM cols WHERE name = $1")
            .bind(metric_name)
            .fetch_one(&self.pool)
            .await?;
        let col_query: String = row.try_get::<String, _>(0)?.into();
        let metric_agg: String = row.try_get::<String, _>(1)?.into();
        Ok((col_query, metric_agg))
    }

    pub async fn get_filter(&self, view_name: String) -> Result<String, sqlx::error::Error> {
        let try_filter = sqlx::query("SELECT query FROM filters WHERE name = $1")
            .bind(view_name)
            .fetch_one(&self.pool)
            .await?;
        Ok(try_filter.try_get::<String, _>(0)?)
    }

    pub async fn get_filters(
        &self,
        start: chrono::DateTime<Utc>,
        end: chrono::DateTime<Utc>,
        metric_agg: String,
        col_query: String,
        where_query: String,
    ) -> Result<Vec<Option<f64>>, sqlx::error::Error> {
        let interval_millis = (end - start).num_milliseconds();
        let interval_micro = (end - start).num_microseconds();
        let interval_str = match interval_micro {
            Some(val) => format!("{} microseconds", max(val / 119, 10)),
            None => format!("{} milliseconds", max(interval_millis / 119, 10)),
        };
        let query = format!(
            "
        SELECT {metric_agg}(({col_query})::numeric)  
            FROM logs 
            WHERE jsonb_typeof({col_query}) = 'number' 
              AND {where_query}
              AND time >= '{}'::TIMESTAMP 
              AND time <= '{}'::TIMESTAMP 
            GROUP BY time_bucket_gapfill('{interval_str}', time) 
            LIMIT 120",
            start, end,
        );
        Ok(sqlx::query(query.as_str())
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| match r.try_get::<BigDecimal, _>(0) {
                Err(_) => None,
                Ok(val) => val.to_f64(),
            })
            .collect::<Vec<Option<f64>>>())
    }

    pub async fn create_mat_views(
        &self,
        filter_name: String,
        filter_query: String,
    ) -> Result<(), sqlx::error::Error> {
        let query = format!(
            "DROP MATERIALIZED VIEW IF EXISTS {filter_name}_sec_count; DROP MATERIALIZED VIEW IF EXISTS {filter_name}_min_count; 
            CREATE MATERIALIZED VIEW {filter_name}_sec_count (time_bucket, count) WITH (timescaledb.continuous)
                AS SELECT time_bucket('1s', time), COUNT(*) from logs where {filter_query} GROUP BY time_bucket('1s', time);
            CREATE MATERIALIZED VIEW {filter_name}_min_count (time_bucket, count) WITH (timescaledb.continuous)
                AS SELECT time_bucket('1 minute', time), COUNT(*) from logs where {filter_query} GROUP BY time_bucket('1 minute', time);
            SELECT add_continuous_aggregate_policy('{filter_name}_min_count',
                start_offset => null,
                end_offset => null,
                schedule_interval => INTERVAL '10 minute');
            SELECT add_continuous_aggregate_policy('{filter_name}_sec_count',
                start_offset => null,
                end_offset => null,
                schedule_interval => INTERVAL '10 seconds');",
        );
        let _res = sqlx::raw_sql(query.as_str()).execute_many(&self.pool);
        Ok(())
    }

    pub async fn delete_view(&self, view_name: String) -> Result<(), sqlx::error::Error> {
        let query = format!(
            "
            DELETE FROM filter WHERE name = {view_name}; 
            DELETE FROM column_filter WHERE filter_name = {view_name}; 
            DELETE FROM cols WHERE name NOT IN (SELECT column_name FROM column_filters);"
        );
        let _res = sqlx::raw_sql(&query.as_str()).execute_many(&self.pool);
        Ok(())
    }

    pub async fn upsert_columns_and_filters(
        &self,
        column_names: &Vec<String>,
        columns_queries: &Vec<(String, String)>,
        filter_name: &String,
        filter_query: &String,
    ) -> Result<(), sqlx::error::Error> {
        let values: Vec<String> = column_names
            .into_iter()
            .zip(columns_queries)
            .map(|(name, (query, metric_agg))| {
                [
                    "('",
                    name,
                    "','",
                    &query.replace("'", "''"),
                    "','",
                    &metric_agg,
                    "')",
                ]
                .join("")
            })
            .collect();
        let query =format!(
                    "INSERT INTO cols (name, query, metric_agg) VALUES {} ON CONFLICT (name) DO UPDATE SET query = EXCLUDED.query",
                    values.join(","));
        let _res = sqlx::query(query.as_str()).execute(&self.pool).await;

        let _res = sqlx::query("DELETE FROM column_filter WHERE filter_name = $1")
            .bind(filter_name)
            .execute(&self.pool)
            .await;
        let filter_column_values: Vec<String> = column_names
            .into_iter()
            .enumerate()
            .map(|(idx, name)| format!("('{}', '{}', {})", name, filter_name, idx))
            .collect();

        let query = format!(
            "INSERT INTO column_filter (column_name, filter_name, idx) VALUES {}",
            filter_column_values.join(",")
        );
        let _res = sqlx::query(&query.as_str()).execute(&self.pool).await;

        let _res = sqlx::query(
                format!(
                    "INSERT INTO filters (name, query) VALUES ('{}', '{}') ON CONFLICT (name) DO UPDATE SET query = EXCLUDED.query",
                    filter_name, filter_query.replace("'", "''")
                ).as_str()).execute(&self.pool).await;
        Ok(())
    }

    pub async fn get_logs(
        &self,
        start: chrono::NaiveDateTime,
        end: chrono::NaiveDateTime,
        offset: i64,
        table: String,
    ) -> Result<Vec<(NaiveDateTime, String, Vec<serde_json::Value>)>, sqlx::error::Error> {
        let query = "
        SELECT COUNT(*), filters.query, array_agg(cols.query ORDER BY idx)
            FROM column_filter
                JOIN filters ON filters.name = column_filter.filter_name
                JOIN cols ON cols.name = column_filter.column_name
            WHERE filters.name = $1
            GROUP BY filters.name, filters.query";
        let row = sqlx::query(query)
            .bind(table)
            .fetch_one(&self.pool)
            .await
            .unwrap();

        let col_number: usize = row.get::<i64, _>(0) as usize;
        let filter_query: String = row.get::<String, _>(1);
        let column_queries: Vec<String> = row.get::<Vec<String>, _>(2);
        let query = format!(
                    "SELECT time, level, {} from logs WHERE {} AND time >= '{}'::TIMESTAMP AND time <= '{}'::TIMESTAMP LIMIT 40 OFFSET {}",
                    column_queries.join(","), filter_query, start, end, offset
                );
        Ok(sqlx::query(query.as_str())
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| {
                let mut ret_line: Vec<serde_json::Value> = Vec::new();
                for _i in 2..col_number + 2 {
                    ret_line.push(match row.try_get::<serde_json::Value, _>(2) {
                        Ok(val) => val,
                        Err(_) => match row.try_get::<f64, _>(2) {
                            Ok(val) => val.into(),
                            Err(_) => match row.try_get::<Vec<String>, _>(2) {
                                Ok(strval) => strval.into(),
                                Err(_) => match row.try_get::<String, _>(2) {
                                    Ok(strval) => strval.into(),
                                    Err(_) => serde_json::from_str("null").unwrap(),
                                },
                            },
                        },
                    })
                }
                (
                    row.get::<NaiveDateTime, _>(0),
                    row.get::<String, _>(1),
                    ret_line,
                )
            })
            .collect())
    }

    pub async fn get_density(
        &self,
        start: chrono::NaiveDateTime,
        end: chrono::NaiveDateTime,
        table: &str,
    ) -> Result<Vec<i64>, sqlx::error::Error> {
        let interval_millis = (end - start).num_milliseconds();
        let interval_micro = (end - start).num_microseconds();
        let interval_str = match interval_micro {
            Some(val) => format!("{} microseconds", max(val / 119, 10)),
            None => format!("{} milliseconds", max(interval_millis / 119, 10)),
        };
        let query = match interval_millis {
            0..=100000 => {
                let where_query = sqlx::query("SELECT query from filters WHERE name = $1")
                    .bind(table)
                    .fetch_one(&self.pool)
                    .await?
                    .try_get::<String, _>(0)?;
                format!(
                    "
                SELECT COUNT(*)::bigint
                    FROM logs
                    WHERE {}
                      AND time >= '{}'::TIMESTAMP
                      AND time <= '{}'::TIMESTAMP
                    GROUP BY time_bucket_gapfill('{}', time)
                    LIMIT 120",
                    where_query, start, end, interval_str
                )
            }
            100001..=10000000 => {
                format!(
                    "
                SELECT sum(count)::bigint
                    FROM {}_sec_count
                    WHERE time_bucket >= '{}'::TIMESTAMP
                      AND time_bucket <= '{}'::TIMESTAMP
                    GROUP BY time_bucket_gapfill('{}', time_bucket)
                    LIMIT 120",
                    table, start, end, interval_str
                )
            }
            _ => {
                format!(
                    "
                SELECT sum(count)::bigint
                    FROM {}_min_count
                    WHERE time_bucket >= '{}'::TIMESTAMP
                      AND time_bucket <= '{}'::TIMESTAMP
                    GROUP BY time_bucket_gapfill('{}', time_bucket)
                    LIMIT 120",
                    table, start, end, interval_str
                )
            }
        };
        Ok(sqlx::query(query.as_str())
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| row.try_get::<i64, _>(0).unwrap_or(0).into())
            .collect())
    }
}
