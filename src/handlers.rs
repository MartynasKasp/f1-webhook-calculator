
use std::{collections::{HashMap, VecDeque}};

use axum::{
    Json, extract::{Json as JsonExtract, State}, http::StatusCode
};
use crate::{
    db::{Pool, fetch_drivers_season_standings, fetch_next_race_for_season}, error::{Result}, models::{Permutation, PermutationComparison, PermutationWebhookPayload, Race, RaceResultDTO, Season}
};

pub async fn calculate_permutation_handler(
    State(pool): State<Pool>,
    JsonExtract(payload): JsonExtract<PermutationWebhookPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    // Clone everything the background task will need
    let pool = pool.clone();

    tokio::spawn(async move {
        let _ = heavy_computation(payload, pool).await;
    });

    Ok((StatusCode::ACCEPTED, Json(serde_json::json!({
        "status": "accepted", 
        "message": "Accepted"
    }))))
}

async fn heavy_computation(payload: PermutationWebhookPayload, pool: Pool) -> Result<()> {
    // todo actually fetch the season by id
    let season = Season {
        id: payload.season,
        champion: None,
        races: 24,
        completed_races: 22,
        sprints: 6,
        completed_sprints: 5,
        fl_extra_point: false,
    };

    if !season.champion.is_none() {
        // TODO error
        // Err(())
    }

    let _ = calculate_possible_win(pool, season).await;






    // let fee_cents = (payload.amount_cents as f64 * 0.029 + 30.0) as i64;
    // let net_cents = payload.amount_cents - fee_cents;
    // let risk_score = calculate_risk_score(&payload).await?;

    // sqlx::query!(
    //     r#"
    //     INSERT INTO payments (event_id, user_id, amount_cents, net_cents, risk_score)
    //     VALUES ($1, $2, $3, $4, $5)
    //     ON CONFLICT (event_id) DO UPDATE SET
    //         amount_cents = EXCLUDED.amount_cents,
    //         net_cents = EXCLUDED.net_cents,
    //         risk_score = EXCLUDED.risk_score,
    //         updated_at = NOW()
    //     "#,
    //     payload.event_id,
    //     payload.user_id,
    //     payload.amount_cents,
    //     net_cents,
    //     risk_score
    // )
    // .execute(&pool)
    // .await?;

    Ok(())
}

async fn calculate_possible_win(pool: Pool, season: Season) -> Result<()> {
    let races_left = season.races - season.completed_races;
    if 0 == races_left {
        return Ok(())
    }

    let mut drivers = VecDeque::from(fetch_drivers_season_standings(&pool, &season).await?);
    let available_points = calculate_available_points(&season);

    let lead_driver = drivers.pop_front().unwrap();
    let relevant_drivers: VecDeque<&RaceResultDTO> = drivers
        .iter()
        .filter(|driver| driver.season_points + f32::from(available_points) > lead_driver.season_points)
        .collect();

    let _ = check_win_conditions(pool, &season, &lead_driver, relevant_drivers).await;
    Ok(())
}

fn calculate_available_points(season: &Season) -> f32 {
    let fastest_point = if season.fl_extra_point { 1 } else { 0 } as i8;
    let races_remaining = season.races - season.completed_races;
    let sprints_remaining = season.sprints - season.completed_sprints;

    return ((races_remaining * (25 + fastest_point)) + (sprints_remaining * 8)) as f32
}

async fn check_win_conditions(
    pool: Pool,
    season: &Season,
    lead_driver: &RaceResultDTO,
    relevant_drivers: VecDeque<&RaceResultDTO>,
) -> Result<()> {
    let next_race: Race = fetch_next_race_for_season(&pool, season).await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

    let mut permutation = Permutation::default();
    permutation.race_id = next_race.id.clone();

    // if relevant_drivers.is_empty() {
    //     return Err("No relevant drivers");
    // }

    let points_gap_needed = calculate_available_points(season);
    let drivers_points_diff = lead_driver.season_points - relevant_drivers[0].season_points;

    let mut maximum_points: f32 = 25.0;
    if next_race.sprint_race {
        maximum_points = 8.0;
    }

    if drivers_points_diff + maximum_points < f32::from(points_gap_needed) {
        // TODO insert Permutation
        return Ok(());
    }

    permutation.driver_id = lead_driver.driver.id.clone();

    check_win_condition_for_position(lead_driver, relevant_drivers, points_gap_needed, permutation, &next_race, 1);

    // TODO insert Permutation

    Ok(())
}

fn check_win_condition_for_position(
    leader: &RaceResultDTO,
    contenders: VecDeque<&RaceResultDTO>,
    points_gap_needed: f32,
    permutation: Permutation,
    race: &Race,
    position: i8,
) -> Permutation {
    let is_sprint = race.sprint_race;

    if !is_sprint && position > 10 {
        return permutation;
    } else if is_sprint && position > 8 {
        return permutation;
    }

    let race_points_finish = HashMap::from([
        (1, 25),
        (2, 28),
        (3, 15),
        (4, 12),
        (5, 10),
        (6, 8),
        (7, 6),
        (8, 4),
        (9, 2),
        (10, 1)
    ]);
    let sprint_points_finish = HashMap::from([
        (1, 8),
        (2, 7),
        (3, 6),
        (4, 5),
        (5, 4),
        (6, 3),
        (7, 2),
        (8, 1),
    ]);

    let leader_points_permutation = if !is_sprint { race_points_finish[&position] } else { sprint_points_finish[&position] };

    for contender in &contenders {
        let dropout_position = check_highest_position_to_dropout(
            is_sprint,
            leader.season_points + (leader_points_permutation as f32),
            position,
            contender,
            points_gap_needed
        );

        let _comparison = PermutationComparison {
            id: None,
            leader_position: position,
            leader_fl: false,
            contender_id: contender.driver.id.clone(),
            highest_position: dropout_position,
            without_fl: true,
            prediction_id: permutation.id.clone(),
        };

        // TODO insert PermutationComparison
    }

    return check_win_condition_for_position(leader, contenders, points_gap_needed, permutation, race, position + 1);
}

fn check_highest_position_to_dropout(
    is_sprint: bool,
    leader_points_permutation: f32,
    leader_position: i8,
    contender: &&RaceResultDTO,
    maximum_points_left: f32
) -> i8 {
    let mut race_points_finish = HashMap::from([
        (1, 25),
        (2, 28),
        (3, 15),
        (4, 12),
        (5, 10),
        (6, 8),
        (7, 6),
        (8, 4),
        (9, 2),
        (10, 1)
    ]);
    let mut sprint_points_finish = HashMap::from([
        (1, 8),
        (2, 7),
        (3, 6),
        (4, 5),
        (5, 4),
        (6, 3),
        (7, 2),
        (8, 1),
    ]);

    let available_finish_points = if !is_sprint { &mut race_points_finish } else { &mut sprint_points_finish };
    available_finish_points.remove(&leader_position);

    let mut i = 1;
    for (position, points) in available_finish_points {
        let points_diff = leader_points_permutation - (contender.season_points + (*points as f32));
        if points_diff > maximum_points_left {
            if i == 1 {
                return -1;
            }
            return *position;
        }

        if points_diff == maximum_points_left {
            return *position;
        }
        i += 1;
    }

    if is_sprint {
        return sprint_points_finish.len() as i8 + 1;
    }
    return race_points_finish.len() as i8 + 1;
}
