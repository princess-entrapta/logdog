use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct LogQuery {
    pub start: chrono::DateTime<Utc>,
    pub end: chrono::DateTime<Utc>,
    #[serde(default = "default_table")]
    pub table: String,
    #[serde(default = "default_offset")]
    pub offset: i64,
}

fn default_offset() -> i64 {
    0
}

fn default_filter() -> String {
    "true".to_owned()
}

fn default_table() -> String {
    "logs".to_owned()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ColumnDef {
    pub name: String,
    pub query: String,
    pub metric_agg: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilterDef {
    pub name: String,
    #[serde(default = "default_filter")]
    pub query: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ViewQuery {
    pub columns: Vec<ColumnDef>,
    pub filter: FilterDef,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MetricQuery {
    pub start: chrono::DateTime<Utc>,
    pub end: chrono::DateTime<Utc>,
    pub metric_name: String,
    pub view_name: String,
}
