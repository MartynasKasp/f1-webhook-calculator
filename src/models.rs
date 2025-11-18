use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct CalculatorWebhookPayload {
    pub grand_prix: i32,
}
