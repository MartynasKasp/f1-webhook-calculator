use serde::{ Deserialize };
use sqlx::{ FromRow,
    types::time::{ PrimitiveDateTime, Date }
};

#[derive(Deserialize, Debug)]
pub struct PermutationWebhookPayload {
    pub season: String,
}

#[derive(FromRow)]
pub struct Driver {
    pub id: String,
    pub number: String,
    pub full_name: String,
    pub team_id: Option<String>,
}

#[derive(FromRow)]
pub struct Season {
    pub id: String,
    pub champion: Option<Driver>,
    pub races: i8,
    pub completed_races: i8,
    pub sprints: i8,
    pub completed_sprints: i8,
    pub fl_extra_point: bool,
}

#[derive(FromRow)]
pub struct Race {
    pub id: String,
    pub date: Date,
    pub completed: bool,
    pub canceled: bool,
    pub full_distance: bool,
    pub sprint_race: bool,
    pub season_id: Option<String>,
    pub grand_prix: String,
    pub circuit_id: Option<String>,
}

#[derive(FromRow)]
pub struct RaceResult {
    pub id: String,
    pub season: Season,
    pub driver: Option<Driver>,
    pub position: i8,
    pub points: f32,
    pub result_status: String,
    pub race: Option<Race>
}

#[derive(FromRow)]
pub struct Circuit {
    pub id: String,
    pub country: String,
    pub circuit: String,
    pub description: String
}

#[derive(FromRow, Default)]
pub struct Permutation {
    pub id: String,
    pub driver_id: String,
    pub race_id: String,
    pub created_at: Option<PrimitiveDateTime>
}

#[derive(FromRow, Default)]
pub struct PermutationComparison {
    pub id: Option<String>,
    pub leader_position: i8,
    pub leader_fl: bool,
    pub highest_position: i8,
    pub without_fl: bool,
    pub contender_id: String,
    pub prediction_id: String,
}

pub struct RaceResultDTO {
    pub driver: Driver,
    pub season_points: f32,
    pub diff_to_leader: f32,
}