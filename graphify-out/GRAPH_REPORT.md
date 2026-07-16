# Graph Report - /Users/fmolliet/workspace/POCS/fbrustapi  (2026-07-15)

## Corpus Check
- Corpus is ~8,768 words - fits in a single context window. You may not need a graph.

## Summary
- 152 nodes · 394 edges · 15 communities
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Integration Tests
- Birthday Routes
- Birthday Models
- Registration Domain
- Dev Scripts
- Registration Routes
- Configuration
- Database Layer

## God Nodes (most connected - your core abstractions)
1. `create_app()` - 25 edges
2. `setup_test_pool()` - 24 edges
3. `seed_schema()` - 24 edges
4. `truncate_tables()` - 24 edges
5. `AppError` - 19 edges
6. `AppState` - 18 edges
7. `Birthday` - 10 edges
8. `Register` - 10 edges
9. `update_birthday()` - 10 edges
10. `finish_registration()` - 10 edges

## Surprising Connections (you probably didn't know these)
- `create_register()` --references--> `AppState`  [EXTRACTED]
  src/routes/register.rs → src/db.rs
- `delete_registration()` --references--> `AppState`  [EXTRACTED]
  src/routes/register.rs → src/db.rs
- `finish_registration()` --references--> `AppState`  [EXTRACTED]
  src/routes/register.rs → src/db.rs
- `get_open_registration()` --references--> `AppState`  [EXTRACTED]
  src/routes/register.rs → src/db.rs
- `get_registration()` --references--> `AppState`  [EXTRACTED]
  src/routes/register.rs → src/db.rs

## Import Cycles
- None detected.

## Communities (15 total, 0 thin omitted)

### Community 0 - "Integration Tests"
Cohesion: 0.28
Nodes (29): create_app(), PgPool, Router, seed_schema(), setup_test_pool(), setup_tracing(), test_create_birthday_returns_201(), test_create_duplicate_birthday_returns_400() (+21 more)

### Community 1 - "Birthday Routes"
Cohesion: 0.21
Nodes (24): Error, Query, Response, AppState, AppError, IntoResponse, String, birthday_routes() (+16 more)

### Community 2 - "Birthday Models"
Cohesion: 0.22
Nodes (9): BirthdayRequestDto, String, Birthday, BirthdayResponse, Option, PgPool, Result, String (+1 more)

### Community 3 - "Registration Domain"
Cohesion: 0.22
Nodes (11): NaiveDateTime, CreateRegisterDto, Option, String, UpdateRegisterDto, Register, Option, PgPool (+3 more)

### Community 4 - "Dev Scripts"
Cohesion: 0.49
Nodes (13): check_docker(), clean_container(), log_error(), log_info(), log_success(), log_warn(), print_usage(), reset_db() (+5 more)

### Community 5 - "Registration Routes"
Cohesion: 0.40
Nodes (13): create_register(), delete_registration(), finish_registration(), get_open_registration(), get_registration(), register_routes(), IntoResponse, Json (+5 more)

### Community 8 - "Configuration"
Cohesion: 0.40
Nodes (3): Self, Config, String

### Community 9 - "Database Layer"
Cohesion: 0.83
Nodes (3): create_pool(), health_check(), PgPool

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `BirthdayRequestDto` connect `Birthday Models` to `Birthday Routes`?**
  _High betweenness centrality (0.159) - this node is a cross-community bridge._
- **Why does `AppState` connect `Birthday Routes` to `Database Layer`, `Registration Routes`?**
  _High betweenness centrality (0.157) - this node is a cross-community bridge._
- **Why does `AppError` connect `Birthday Routes` to `Registration Routes`?**
  _High betweenness centrality (0.156) - this node is a cross-community bridge._