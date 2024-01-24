use serde::Serialize;

#[derive(Serialize)]
pub struct AuthenticationResponse {
    pub token: String,
}

#[derive(Serialize)]
pub struct OrderSubmissionResponse {
    pub token: String,
}