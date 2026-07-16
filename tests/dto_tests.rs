use fbrustapi::dto::register::{CreateRegisterDto, UpdateRegisterDto};
use fbrustapi::dto::birthday::BirthdayRequestDto;
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
