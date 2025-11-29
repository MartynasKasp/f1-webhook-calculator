use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

use crate::models::{Driver, Race, RaceResultDTO, Season};

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await
}

pub async fn fetch_driver(pool: &Pool, id: &String) -> Result<Driver, sqlx::Error> {
    sqlx::query_as!(
        Driver,
        "SELECT * FROM driver WHERE id = $1",
        id
    )
    .fetch_one(pool)
    .await
}

pub async fn fetch_drivers_season_standings(pool: &Pool, season: &Season) -> Result<Vec<RaceResultDTO>, sqlx::Error> {
    let results = sqlx::query!(
        r#"
            SELECT
                COALESCE(SUM(rr.points), 0) AS "season_points!",
                rr.driver_id AS "driver_id!"
            FROM race_result rr
            WHERE rr.season_id = $1
            GROUP BY driver_id
            ORDER BY COALESCE(SUM(rr.points), 0) DESC
        "#,
        season.id
    )
    .fetch_all(pool)
    .await?;

    let mut standings: Vec<RaceResultDTO> = vec![];
    for row in &results {
        let diff = results[0].season_points - row.season_points;

        standings.push(RaceResultDTO {
            driver: fetch_driver(pool, &row.driver_id).await?,
            season_points: row.season_points as f32,
            diff_to_leader: diff as f32
        });
    }

    Ok(standings)
}

pub async fn fetch_next_race_for_season(pool: &Pool, season: &Season) -> Result<Option<Race>, sqlx::Error> {
    sqlx::query_as!(
        Race,
        r#"
            SELECT * FROM race r
            WHERE
                r.completed = FALSE
                AND r.season_id = $1
            ORDER BY r.date ASC
            LIMIT 1
        "#,
        season.id
    )
    .fetch_optional(pool)
    .await
}

pub type Pool = PgPool;
