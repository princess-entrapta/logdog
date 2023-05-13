use std::{cmp::max, sync::Arc};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::NaiveDateTime;

use crate::{
    model::{LogQuery, ViewQuery},
    AppState,
};

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Log viewer utility";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn upsert_columns_and_filters(
    data: &Arc<AppState>,
    column_names: &Vec<String>,
    columns_queries: &Vec<String>,
    filter_name: &String,
    filter_query: &String,
) -> Result<(), tokio_postgres::Error> {
    let client = data.db.get().await.unwrap();
    let values: Vec<String> = column_names
        .into_iter()
        .zip(columns_queries)
        .map(|(name, query)| ["('", name, "','", &query.replace("'", "''"), "')"].join(""))
        .collect();
    let _res = client
        .query(
            &format!(
                "INSERT INTO cols (name, query) VALUES {} ON CONFLICT (name) DO UPDATE SET query = EXCLUDED.query",
                values.join(",")
            ),
            &[],
        )
        .await?;

    let _res = client
        .query(
            "DELETE FROM column_filter WHERE filter_name = $1",
            &[filter_name],
        )
        .await?;
    let filter_column_values: Vec<String> = column_names
        .into_iter()
        .enumerate()
        .map(|(idx, name)| format!("('{}', '{}', {})", name, filter_name, idx))
        .collect();
    let _res = client
        .query(
            &format!(
                "INSERT INTO column_filter (column_name, filter_name, idx) VALUES {}",
                filter_column_values.join(",")
            ),
            &[],
        )
        .await?;
    let _res = client
        .query(
            &format!(
                "INSERT INTO filters (name, query) VALUES ('{}', '{}') ON CONFLICT (name) DO UPDATE SET query = EXCLUDED.query",
                filter_name, filter_query.replace("'", "''")
            ),
            &[],
        )
        .await?;
    Ok(())
}

pub async fn create_view(
    data: &Arc<AppState>,
    columns_queries: Vec<String>,
    column_names: Vec<String>,
    filter_name: String,
    filter_query: String,
) -> Result<(StatusCode, String), tokio_postgres::Error> {
    upsert_columns_and_filters(
        data,
        &column_names,
        &columns_queries,
        &filter_name,
        &filter_query,
    )
    .await?;
    let client = data.db.get().await.unwrap();
    let _res = client
        .batch_execute(&format!(
            "DROP MATERIALIZED VIEW IF EXISTS {}_sec_count; DROP MATERIALIZED VIEW IF EXISTS {}_min_count; ",
            filter_name, filter_name
        ))
        .await?;
    let _res = client
    .query(
        &format!(
            "CREATE MATERIALIZED VIEW {}_sec_count (time_bucket, count) WITH (timescaledb.continuous) AS SELECT time_bucket('1s', time), COUNT(*) from logs where {} GROUP BY time_bucket('1s', time)",
            filter_name, filter_query,
        ),
        &[],
    )
    .await?;
    let _res = client
    .query(
        &format!(
            "CREATE MATERIALIZED VIEW {}_min_count (time_bucket, count) WITH (timescaledb.continuous) AS SELECT time_bucket('1 minute', time), COUNT(*) from logs where {} GROUP BY time_bucket('1 minute', time)",
            filter_name, filter_query,
        ),
        &[],
    ).await?;
    let _res = client
        .query(
            &format!(
                "SELECT add_continuous_aggregate_policy('{}_min_count',
            start_offset => null,
            end_offset => null,
            schedule_interval => INTERVAL '10 minute');",
                filter_name,
            ),
            &[],
        )
        .await?;
    let _res = client
        .query(
            &format!(
                "SELECT add_continuous_aggregate_policy('{}_sec_count',
        start_offset => null,
        end_offset => null,
        schedule_interval => INTERVAL '10 seconds');",
                filter_name,
            ),
            &[],
        )
        .await?;
    return Ok((StatusCode::CREATED, "{}".to_string()));
}

pub async fn get_logs(
    data: &Arc<AppState>,
    table: String,
    start: chrono::NaiveDateTime,
    end: chrono::NaiveDateTime,
    offset: i64,
) -> Result<Vec<Vec<serde_json::Value>>, (StatusCode, String)> {
    let client = data.db.get().await.unwrap();

    let row = client
        .query_one(
            "SELECT COUNT(*), filters.query, array_agg(cols.query ORDER BY idx) from column_filter JOIN filters ON filters.name = column_filter.filter_name JOIN cols ON cols.name = column_filter.column_name WHERE filters.name = $1 GROUP BY filters.name, filters.query",
            &[&table],
        )
        .await
        .unwrap();

    let col_number: usize = row.get::<_, i64>(0) as usize;
    let filter_query: String = row.get::<_, String>(1);
    let column_queries: Vec<String> = row.get::<_, Vec<String>>(2);
    let row = client
        .query(
            &format!(
                "SELECT time, level, {} from logs WHERE {} AND time >= '{}'::TIMESTAMP AND time <= '{}'::TIMESTAMP LIMIT 40 OFFSET {}",
                column_queries.join(","), filter_query, start, end, offset
            ),
            &[],
        )
        .await;

    if row.is_err() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            row.unwrap_err().to_string(),
        ));
    }

    let rows = row.unwrap();
    let mut ret_val: Vec<Vec<serde_json::Value>> = Vec::new();
    for r in rows {
        let mut ret_line: Vec<serde_json::Value> = Vec::new();
        ret_line.push(match r.try_get::<_, NaiveDateTime>(0) {
            Ok(val) => val.to_string().into(),
            Err(_) => "".into(),
        });
        ret_line.push(r.get::<_, Option<String>>(1).into());
        for i in 2..col_number + 2 {
            let json_value: serde_json::Value = match r.try_get::<_, serde_json::Value>(i) {
                Ok(val) => val,
                Err(_) => match r.try_get::<_, f64>(i) {
                    Ok(val) => val.into(),
                    Err(_) => match r.try_get::<_, Vec<String>>(i) {
                        Ok(strval) => strval.into(),
                        Err(_) => match r.try_get::<_, String>(i) {
                            Ok(strval) => strval.into(),
                            Err(_) => serde_json::from_str("null").unwrap(),
                        },
                    },
                },
            };
            ret_line.push(json_value);
        }
        ret_val.push(ret_line);
    }
    Ok(ret_val)
}

pub async fn get_density(
    data: &Arc<AppState>,
    table: String,
    start: chrono::NaiveDateTime,
    end: chrono::NaiveDateTime,
) -> Result<Vec<serde_json::Number>, (StatusCode, String)> {
    let client = data.db.get().await.unwrap();
    let interval_millis = (end - start).num_milliseconds();
    let interval_micro = (end - start).num_microseconds();
    let interval_str = match interval_micro {
        Some(val) => format!("{} microseconds", max(val / 80, 10)),
        None => format!("{} milliseconds", max(interval_millis / 80, 10)),
    };
    let row = match interval_millis {
        0..=100000 => client
        .query(
            &format!(
                "SELECT COUNT(*)::bigint from {} WHERE time >= '{}'::TIMESTAMP AND time <= '{}'::TIMESTAMP GROUP BY time_bucket_gapfill('{}', time)",
                table, start, end, interval_str
            ),
            &[],
        )
        .await,
        100001..=10000000 => client.query(
            &format!(
                "SELECT sum(count)::bigint from {}_sec_count WHERE time_bucket >= '{}'::TIMESTAMP AND time_bucket <= '{}'::TIMESTAMP GROUP BY time_bucket_gapfill('{}', time_bucket)",
                table, start, end, interval_str
            ),&[],
        ).await,
        _ => client.query(
            &format!(
                "SELECT sum(count)::bigint from {}_min_count WHERE time_bucket >= '{}'::TIMESTAMP AND time_bucket <= '{}'::TIMESTAMP GROUP BY time_bucket_gapfill('{}', time_bucket)",
                table, start, end, interval_str
            ),
            &[],
        )
        .await,
    };
    if row.is_err() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            row.unwrap_err().to_string(),
        ));
    }

    let rows = row.unwrap();
    let mut ret_val: Vec<serde_json::Number> = Vec::new();
    for r in rows {
        ret_val.push(r.try_get::<_, i64>(0).unwrap_or(0).into());
    }
    Ok(ret_val)
}

pub async fn density_handler(
    State(data): State<Arc<AppState>>,
    density_query: Json<LogQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let logs = match get_density(
        &data,
        density_query.table.to_owned(),
        density_query.start.naive_utc(),
        density_query.end.naive_utc(),
    )
    .await
    {
        Ok(vec) => vec,
        Err((code, json)) => return Err((code, json)),
    };
    return Ok(Json(logs));
}

pub async fn logs_handler(
    State(data): State<Arc<AppState>>,
    log_query: Json<LogQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let logs = match get_logs(
        &data,
        log_query.table.to_owned(),
        log_query.start.naive_utc(),
        log_query.end.naive_utc(),
        log_query.offset,
    )
    .await
    {
        Ok(vec) => vec,
        Err((code, json)) => return Err((code, json)),
    };
    return Ok(Json(logs));
}

pub async fn view_handler(
    State(data): State<Arc<AppState>>,
    log_query: Json<ViewQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let filter_name = log_query.filter.name.to_owned();
    let filter_query = log_query.filter.query.to_owned();
    let (names, queries) = log_query
        .0
        .columns
        .into_iter()
        .map(|c| (c.name, c.query))
        .unzip();
    let filter_name = if filter_name.len() == 0 {
        "logs".to_owned()
    } else {
        filter_name
    };
    match create_view(&data, queries, names, filter_name, filter_query).await {
        Ok(vec) => Ok(vec),
        Err(error) => return Err((StatusCode::INTERNAL_SERVER_ERROR, error.to_string())),
    }
}

pub async fn list_views(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client = data.db.get().await.unwrap();
    match client.query("SELECT filters.name, array_agg(cols.name ORDER BY idx) from filters JOIN column_filter ON filters.name = column_filter.filter_name JOIN cols ON cols.name = column_filter.column_name GROUP BY filters.name;", &[]).await {
        Ok(rows) => Ok(Json(
            rows.into_iter()
                .map(|r| { let mut val = serde_json::Map::new();
            val.insert("name".to_owned(), r.get::<_, String>(0).into());
            val.insert("cols".to_owned(), r.get::<_, Vec<String>>(1).into());
        val.into()}
            )
                .collect::<Vec<serde_json::Value>>(),
        )),
        Err(error) => Err((StatusCode::INTERNAL_SERVER_ERROR, error.to_string())),
    }
}
