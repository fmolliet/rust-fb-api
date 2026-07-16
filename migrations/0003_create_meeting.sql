CREATE TABLE IF NOT EXISTS meeting (
    id          BIGSERIAL PRIMARY KEY,
    name        VARCHAR(200) NOT NULL DEFAULT '',
    snowflake   VARCHAR(50) NOT NULL UNIQUE,
    state       VARCHAR(2) NOT NULL,
    active      BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX IF NOT EXISTS idx_meeting_state_active ON meeting(state, active);
CREATE INDEX IF NOT EXISTS idx_meeting_active ON meeting(active);
