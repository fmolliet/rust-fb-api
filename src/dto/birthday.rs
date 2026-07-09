use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct BirthdayRequestDto {
    #[validate(length(min = 1))]
    pub snowflake: String,
    #[validate(length(min = 1))]
    pub name: String,
    pub day: u8,
    pub month: u8,
}
