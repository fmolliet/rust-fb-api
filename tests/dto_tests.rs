use fbrustapi::dto::register::{CreateRegisterDto, UpdateRegisterDto};
use fbrustapi::dto::birthday::BirthdayRequestDto;
use fbrustapi::dto::meeting::{CreateMeetingDto, UpdateMeetingDto};
use fbrustapi::dto::announce::{CreateAnnouncementDto, UpdateAnnouncementDto};
use validator::Validate;

#[test]
fn test_create_register_dto_valid() {
    let dto = CreateRegisterDto {
        applicant_id: "123".to_string(),
        birthday: Some("2000-01-01".to_string()),
        specie: Some("Human".to_string()),
        source: Some("Discord".to_string()),
        message_id: Some("987654321".to_string()),
    };
    assert!(dto.validate().is_ok());
}

#[test]
fn test_create_register_dto_invalid_empty_fields() {
    let dto = CreateRegisterDto {
        applicant_id: "".to_string(), // Invalid: empty
        birthday: Some("2000-01-01".to_string()),
        specie: Some("".to_string()), // Invalid: empty
        source: Some("Discord".to_string()),
        message_id: Some("".to_string()), // Invalid: empty
    };
    let res = dto.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    let map = errs.field_errors();
    assert!(map.contains_key("applicant_id"));
    assert!(map.contains_key("specie"));
    assert!(map.contains_key("message_id"));
    assert!(!map.contains_key("source"));
}

#[test]
fn test_update_register_dto_valid() {
    let dto = UpdateRegisterDto {
        applicant_id: "123".to_string(),
        curator_id: "admin".to_string(),
        approved: true,
    };
    assert!(dto.validate().is_ok());
}

#[test]
fn test_update_register_dto_invalid() {
    let dto = UpdateRegisterDto {
        applicant_id: "".to_string(),
        curator_id: "".to_string(),
        approved: false,
    };
    let res = dto.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    let map = errs.field_errors();
    assert!(map.contains_key("applicant_id"));
    assert!(map.contains_key("curator_id"));
}

#[test]
fn test_birthday_request_dto_valid() {
    let dto = BirthdayRequestDto {
        snowflake: "123456789".to_string(),
        name: "Alice".to_string(),
        day: 15,
        month: 6,
    };
    assert!(dto.validate().is_ok());
}

#[test]
fn test_birthday_request_dto_invalid() {
    let dto = BirthdayRequestDto {
        snowflake: "".to_string(),
        name: "".to_string(),
        day: 32,
        month: 13,
    };
    let res = dto.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    let map = errs.field_errors();
    assert!(map.contains_key("snowflake"));
    assert!(map.contains_key("name"));
}

#[test]
fn test_create_meeting_dto_valid() {
    let dto = CreateMeetingDto {
        name: Some("Planning".to_string()),
        snowflake: "abc-123".to_string(),
        state: "SP".to_string(),
    };
    assert!(dto.validate().is_ok());
}

#[test]
fn test_create_meeting_dto_valid_empty_name() {
    let dto = CreateMeetingDto {
        name: None,
        snowflake: "abc-123".to_string(),
        state: "RJ".to_string(),
    };
    assert!(dto.validate().is_ok());
}

#[test]
fn test_create_meeting_dto_invalid_empty_snowflake() {
    let dto = CreateMeetingDto {
        name: Some("Meeting".to_string()),
        snowflake: "".to_string(),
        state: "SP".to_string(),
    };
    let res = dto.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    let map = errs.field_errors();
    assert!(map.contains_key("snowflake"));
}

#[test]
fn test_create_meeting_dto_invalid_state_length() {
    let dto = CreateMeetingDto {
        name: Some("Meeting".to_string()),
        snowflake: "abc-123".to_string(),
        state: "S".to_string(),
    };
    let res = dto.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    let map = errs.field_errors();
    assert!(map.contains_key("state"));
}

#[test]
fn test_update_meeting_dto_valid() {
    let dto = UpdateMeetingDto {
        name: Some("Updated Meeting".to_string()),
        snowflake: "abc-123".to_string(),
        state: "MG".to_string(),
    };
    assert!(dto.validate().is_ok());
}

#[test]
fn test_update_meeting_dto_valid_no_name() {
    let dto = UpdateMeetingDto {
        name: None,
        snowflake: "abc-123".to_string(),
        state: "SP".to_string(),
    };
    assert!(dto.validate().is_ok());
}

#[test]
fn test_update_meeting_dto_invalid_snowflake() {
    let dto = UpdateMeetingDto {
        name: Some("Meeting".to_string()),
        snowflake: "".to_string(),
        state: "SP".to_string(),
    };
    let res = dto.validate();
    assert!(res.is_err());
}

#[test]
fn test_update_meeting_dto_invalid_state() {
    let dto = UpdateMeetingDto {
        name: Some("Meeting".to_string()),
        snowflake: "abc".to_string(),
        state: "X".to_string(),
    };
    let res = dto.validate();
    assert!(res.is_err());
}

#[test]
fn test_create_announcement_dto_valid() {
    let dto = CreateAnnouncementDto {
        channel_id: "1234567890".to_string(),
        message: "Hello Discord!".to_string(),
        frequency: Some("daily".to_string()),
    };
    assert!(dto.validate().is_ok());
}

#[test]
fn test_create_announcement_dto_valid_no_frequency() {
    let dto = CreateAnnouncementDto {
        channel_id: "1234567890".to_string(),
        message: "Hello Discord!".to_string(),
        frequency: None,
    };
    assert!(dto.validate().is_ok());
}

#[test]
fn test_create_announcement_dto_invalid_empty_channel_id() {
    let dto = CreateAnnouncementDto {
        channel_id: "".to_string(),
        message: "Hello Discord!".to_string(),
        frequency: Some("daily".to_string()),
    };
    let res = dto.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    let map = errs.field_errors();
    assert!(map.contains_key("channel_id"));
}

#[test]
fn test_create_announcement_dto_valid_long_message() {
    let dto = CreateAnnouncementDto {
        channel_id: "1234567890".to_string(),
        message: "a".repeat(10000),
        frequency: Some("daily".to_string()),
    };
    assert!(dto.validate().is_ok());
}

#[test]
fn test_create_announcement_dto_exceeds_message_length() {
    let dto = CreateAnnouncementDto {
        channel_id: "1234567890".to_string(),
        message: "a".repeat(10001),
        frequency: Some("daily".to_string()),
    };
    let res = dto.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    let map = errs.field_errors();
    assert!(map.contains_key("message"));
}

#[test]
fn test_update_announcement_dto_valid() {
    let dto = UpdateAnnouncementDto {
        channel_id: Some("1234567890".to_string()),
        message: Some("Updated message".to_string()),
        frequency: Some("weekly".to_string()),
    };
    assert!(dto.validate().is_ok());
}

#[test]
fn test_update_announcement_dto_valid_all_none() {
    let dto = UpdateAnnouncementDto {
        channel_id: None,
        message: None,
        frequency: None,
    };
    assert!(dto.validate().is_ok());
}

#[test]
fn test_update_announcement_dto_invalid_channel_id_empty() {
    let dto = UpdateAnnouncementDto {
        channel_id: Some("".to_string()),
        message: None,
        frequency: None,
    };
    let res = dto.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    let map = errs.field_errors();
    assert!(map.contains_key("channel_id"));
}

#[test]
fn test_update_announcement_dto_invalid_frequency_too_long() {
    let dto = UpdateAnnouncementDto {
        channel_id: None,
        message: None,
        frequency: Some("a".repeat(51)),
    };
    let res = dto.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    let map = errs.field_errors();
    assert!(map.contains_key("frequency"));
}

