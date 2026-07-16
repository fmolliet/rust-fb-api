use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRegisterDto {
    #[validate(length(min = 1))]
    pub applicant_id: String,
    pub birthday: Option<String>,
    #[validate(length(min = 1))]
    pub specie: Option<String>,
    #[validate(length(min = 1))]
    pub source: Option<String>,
    #[validate(length(min = 1))]
    pub message_id: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRegisterDto {
    #[validate(length(min = 1))]
    pub applicant_id: String,
    #[validate(length(min = 1))]
    pub curator_id: String,
    pub approved: bool,
}
