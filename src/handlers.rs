use axum::{Json, extract::{State, Json as JsonExtract}};
use crate::{models::CalculatorWebhookPayload, db::Pool, Result, error::AppError};

// This is where you do your actual calculation
pub async fn webhook_handler(
    State(pool): State<Pool>,
    JsonExtract(payload): JsonExtract<CalculatorWebhookPayload>,
) -> Result<Json<serde_json::Value>> {
    // Example calculation – replace with your real logic
    let fee_cents = (payload.amount_cents as f64 * 0.029 + 30.0) as i64;
    let net_cents = payload.amount_cents - fee_cents;
    let risk_score = calculate_risk_score(&payload).await?; // your function

    // Example DB write
    // sqlx::query!(
    //     r#"
    //     INSERT INTO payments (event_id, user_id, amount_cents, net_cents, risk_score)
    //     VALUES ($1, $2, $3, $4, $5)
    //     "#,
    //     payload.event_id,
    //     payload.user_id,
    //     payload.amount_cents,
    //     net_cents,
    //     risk_score
    // )
    // .execute(&pool)
    // .await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "received_event_id": payload.event_id,
        "calculated_net_cents": net_cents,
        "risk_score": risk_score
    })))
}

async fn calculate_risk_score(_payload: &CalculatorWebhookPayload) -> Result<i32> {
    // put your real calculation here
    Ok(42)
}
