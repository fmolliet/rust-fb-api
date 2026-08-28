use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAnnouncementDto {
    #[validate(length(min = 1, max = 255))]
    pub channel_id: Option<String>,
    #[validate(length(max = 10000))]
    pub message: Option<String>,
    #[validate(length(max = 50))]
    pub frequency: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAnnouncementDto {
    #[validate(length(min = 1, max = 255))]
    pub channel_id: Option<String>,
    #[validate(length(max = 10000))]
    pub message: Option<String>,
    #[validate(length(max = 50))]
    pub frequency: Option<String>,
}
