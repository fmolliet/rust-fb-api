use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMeetingDto {
    #[validate(length(max = 200))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub snowflake: String,
    #[validate(length(equal = 2))]
    pub state: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMeetingDto {
    #[validate(length(max = 200))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub snowflake: String,
    #[validate(length(equal = 2))]
    pub state: String,
}
