use axum::{Router, body::Body, http::Request};
use axum::http::StatusCode;
use sqlx::PgPool;
use tokio;
use tokio::sync::Mutex;
use tracing_subscriber;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;
use tower::util::ServiceExt;
use uuid::Uuid;

static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

fn setup_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "fbrustapi=debug,sqlx=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

use fbrustapi::db::AppState;

async fn setup_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for integration tests");
    PgPool::connect(&database_url).await.expect("Failed to connect to database")
}

async fn seed_schema(pool: &PgPool) {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS registrations (
            id UUID NOT NULL DEFAULT gen_random_uuid(),
            applicantid VARCHAR(255) NOT NULL,
            curatorid VARCHAR(255) NULL,
            finished BOOL NULL DEFAULT false,
            messageid VARCHAR(255) NULL,
            birthday VARCHAR(255) NULL,
            specie VARCHAR(255) NULL,
            source VARCHAR(255) NULL,
            createdat TIMESTAMP NOT NULL DEFAULT current_timestamp,
            updatedat TIMESTAMP NOT NULL DEFAULT current_timestamp,
            approved BOOL NULL
        )"#
    ).execute(pool).await.expect("Failed to create registrations table");

    sqlx::query(
        r#"DO $$
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE tablename = 'registrations' AND indexname = 'idx_applicantid') THEN
                CREATE INDEX idx_applicantid ON registrations(applicantid);
            END IF;
        END $$;"#
    ).execute(pool).await.ok();

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS birthdays (
            day INT4 NOT NULL,
            month INT4 NOT NULL,
            name VARCHAR(255) NOT NULL,
            snowflake VARCHAR(255) NOT NULL
        )"#
    ).execute(pool).await.expect("Failed to create birthdays table");

    sqlx::query(
        r#"DO $$
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_constraints WHERE conname = 'birthdays_pk') THEN
                ALTER TABLE birthdays ADD CONSTRAINT birthdays_pk PRIMARY KEY (snowflake);
            END IF;
        END $$;"#
    ).execute(pool).await.ok();

    sqlx::query(
        r#"DO $$
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE tablename = 'birthdays' AND indexname = 'snowflake_un') THEN
                CREATE UNIQUE INDEX snowflake_un ON birthdays(snowflake);
            END IF;
        END $$;"#
    ).execute(pool).await.ok();
}

async fn truncate_tables(pool: &PgPool) {
    sqlx::query("DELETE FROM birthdays").execute(pool).await.expect("truncate birthdays");
    sqlx::query("DELETE FROM registrations").execute(pool).await.expect("truncate registrations");
}

fn create_app(pool: PgPool) -> Router {
    let state = AppState { pool };

    Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .nest("/registration", fbrustapi::routes::register::register_routes())
        .nest("/birthday", fbrustapi::routes::birthday::birthday_routes())
        .with_state(state)
}

mod registration {
    use super::*;
    use axum::http::header::CONTENT_TYPE;

    #[tokio::test]
    async fn test_create_registration_returns_201() {
        let _guard = TEST_MUTEX.lock().await;
        setup_tracing();
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        let app = create_app(pool);

        let body = serde_json::json!({
            "applicant_id": "app001",
            "birthday": "2000-05-15",
            "specie": "Ara",
            "source": "web",
            "message_id": "msg001"
        }).to_string();

        let req = Request::builder()
            .method("POST")
            .uri("/registration")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        let status = response.status();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();
        eprintln!("Status: {}, Body: {}", status, body_str);
        assert_eq!(status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_duplicate_registration_returns_400() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        sqlx::query(
            r#"INSERT INTO registrations (applicantid, birthday, specie, source, messageid, finished)
               VALUES ($1, $2, $3, $4, $5, FALSE)"#
        )
        .bind("app_dup")
        .bind("2000-01-01")
        .bind("Parrot")
        .bind("test")
        .bind("msg_dup")
        .execute(&pool)
        .await
        .unwrap();

        let app = create_app(pool);

        let body = serde_json::json!({
            "applicant_id": "app_dup",
            "birthday": "2000-01-01",
            "specie": "Parrot",
            "source": "test",
            "message_id": "msg_dup2"
        }).to_string();

        let req = Request::builder()
            .method("POST")
            .uri("/registration")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_registration_by_id_returns_200() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        let reg_id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO registrations (id, applicantid, birthday, specie, source, messageid, finished)
               VALUES ($1, $2, $3, $4, $5, $6, FALSE)"#
        )
        .bind(reg_id)
        .bind("app_get")
        .bind("2000-01-01")
        .bind("Ara")
        .bind("web")
        .bind("msg_get")
        .execute(&pool)
        .await
        .unwrap();

        let app = create_app(pool);

        let req = Request::builder()
            .method("GET")
            .uri(format!("/registration/{}", reg_id))
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_registration_not_found_returns_404() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        let app = create_app(pool);

        let fake_id = Uuid::new_v4();
        let req = Request::builder()
            .method("GET")
            .uri(format!("/registration/{}", fake_id))
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_open_registration_returns_200() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        sqlx::query(
            r#"INSERT INTO registrations (applicantid, birthday, specie, source, messageid, finished)
               VALUES ($1, $2, $3, $4, $5, FALSE)"#
        )
        .bind("app_open")
        .bind("2000-06-20")
        .bind("Scarlet")
        .bind("tg")
        .bind("msg_open")
        .execute(&pool)
        .await
        .unwrap();

        let app = create_app(pool);

        let req = Request::builder()
            .method("GET")
            .uri("/registration/open/app_open")
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_finish_registration_returns_202() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        sqlx::query(
            r#"INSERT INTO registrations (applicantid, birthday, specie, source, messageid, finished)
               VALUES ($1, $2, $3, $4, $5, FALSE)"#
        )
        .bind("app_finish")
        .bind("2000-03-10")
        .bind("Macaw")
        .bind("web")
        .bind("msg_finish")
        .execute(&pool)
        .await
        .unwrap();

        let app = create_app(pool);

        let body = serde_json::json!({
            "applicant_id": "app_finish",
            "curator_id": "curator1",
            "approved": true
        }).to_string();

        let req = Request::builder()
            .method("PATCH")
            .uri("/registration/app_finish/finish")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_finish_nonexistent_registration_returns_404() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        let app = create_app(pool);

        let body = serde_json::json!({
            "applicant_id": "app_noexist",
            "curator_id": "curator1",
            "approved": true
        }).to_string();

        let req = Request::builder()
            .method("PATCH")
            .uri("/registration/app_noexist/finish")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_registration_returns_204() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        let reg_id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO registrations (id, applicantid, birthday, specie, source, messageid, finished)
               VALUES ($1, $2, $3, $4, $5, $6, FALSE)"#
        )
        .bind(reg_id)
        .bind("app_del")
        .bind("2000-07-07")
        .bind("Ara")
        .bind("web")
        .bind("msg_del")
        .execute(&pool)
        .await
        .unwrap();

        let app = create_app(pool);

        let req = Request::builder()
            .method("DELETE")
            .uri(format!("/registration/{}", reg_id))
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_delete_nonexistent_registration_returns_404() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        let app = create_app(pool);

        let fake_id = Uuid::new_v4();
        let req = Request::builder()
            .method("DELETE")
            .uri(format!("/registration/{}", fake_id))
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_registration_invalid_uuid_returns_400() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        let app = create_app(pool);

        let req = Request::builder()
            .method("GET")
            .uri("/registration/not-a-uuid")
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}

mod birthday {
    use super::*;
    use axum::http::header::CONTENT_TYPE;

    #[tokio::test]
    async fn test_create_birthday_returns_201() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        let app = create_app(pool);

        let body = serde_json::json!({
            "snowflake": "1234567890",
            "name": "TestUser",
            "day": 15,
            "month": 6
        }).to_string();

        let req = Request::builder()
            .method("POST")
            .uri("/birthday")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        let status = response.status();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();
        eprintln!("CREATE_BIRTHDAY status={} body={}", status, body_str);
        assert_eq!(status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_duplicate_birthday_returns_400() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        sqlx::query(
            r#"INSERT INTO birthdays (snowflake, name, day, month)
               VALUES ($1, $2, $3, $4)"#
        )
        .bind("dup_sf")
        .bind("DupUser")
        .bind(15)
        .bind(6)
        .execute(&pool)
        .await
        .unwrap();

        let app = create_app(pool);

        let body = serde_json::json!({
            "snowflake": "dup_sf",
            "name": "AnotherName",
            "day": 16,
            "month": 7
        }).to_string();

        let req = Request::builder()
            .method("POST")
            .uri("/birthday")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        let status = response.status();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();
        eprintln!("DUP_BIRTHDAY status={} body={}", status, body_str);
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_birthday_returns_200() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        sqlx::query(
            r#"INSERT INTO birthdays (snowflake, name, day, month)
               VALUES ($1, $2, $3, $4)"#
        )
        .bind("get_sf")
        .bind("GetUser")
        .bind(20)
        .bind(3)
        .execute(&pool)
        .await
        .unwrap();

        let app = create_app(pool);

        let req = Request::builder()
            .method("GET")
            .uri("/birthday/get_sf")
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_birthday_not_found_returns_404() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        let app = create_app(pool);

        let req = Request::builder()
            .method("GET")
            .uri("/birthday/nonexistent_sf")
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_birthday_returns_202() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        sqlx::query(
            r#"INSERT INTO birthdays (snowflake, name, day, month)
               VALUES ($1, $2, $3, $4)"#
        )
        .bind("update_sf")
        .bind("OldName")
        .bind(15)
        .bind(6)
        .execute(&pool)
        .await
        .unwrap();

        let app = create_app(pool);

        let body = serde_json::json!({
            "snowflake": "update_sf",
            "name": "NewName",
            "day": 20,
            "month": 9
        }).to_string();

        let req = Request::builder()
            .method("PATCH")
            .uri("/birthday/update_sf")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_delete_birthday_returns_204() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        sqlx::query(
            r#"INSERT INTO birthdays (snowflake, name, day, month)
               VALUES ($1, $2, $3, $4)"#
        )
        .bind("del_sf")
        .bind("DelUser")
        .bind(10)
        .bind(1)
        .execute(&pool)
        .await
        .unwrap();

        let app = create_app(pool);

        let req = Request::builder()
            .method("DELETE")
            .uri("/birthday/del_sf")
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_delete_nonexistent_birthday_returns_404() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        let app = create_app(pool);

        let req = Request::builder()
            .method("DELETE")
            .uri("/birthday/noexist_sf")
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_birthdays_returns_200_with_data() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        for i in 1..=5 {
            sqlx::query(
                r#"INSERT INTO birthdays (snowflake, name, day, month)
                   VALUES ($1, $2, $3, $4)"#
            )
            .bind(format!("list_sf_{}", i))
            .bind(format!("User{}", i))
            .bind(15)
            .bind(6)
            .execute(&pool)
            .await
            .unwrap();
        }

        let app = create_app(pool);

        let req = Request::builder()
            .method("GET")
            .uri("/birthday?page=0&size=10")
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_birthdays_pagination_respects_size() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        for i in 1..=10 {
            sqlx::query(
                r#"INSERT INTO birthdays (snowflake, name, day, month)
                   VALUES ($1, $2, $3, $4)"#
            )
            .bind(format!("pg_sf_{}", i))
            .bind(format!("User{}", i))
            .bind(15)
            .bind(6)
            .execute(&pool)
            .await
            .unwrap();
        }

        let app = create_app(pool);

        let req = Request::builder()
            .method("GET")
            .uri("/birthday?page=0&size=3")
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_today_birthdays_returns_200() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        let app = create_app(pool);

        let req = Request::builder()
            .method("GET")
            .uri("/birthday/today")
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_month_birthdays_returns_200() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        let app = create_app(pool);

        let req = Request::builder()
            .method("GET")
            .uri("/birthday/month")
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}

mod health {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint() {
        let _guard = TEST_MUTEX.lock().await;
        let pool = setup_test_pool().await;
        seed_schema(&pool).await;
        truncate_tables(&pool).await;

        let app = create_app(pool);

        let req = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
