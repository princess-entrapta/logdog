use std::error::Error;

use sqlx::{postgres::PgRow, PgPool, Row};

#[derive(Clone)]
pub struct Repository {
    pool: PgPool,
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
}
