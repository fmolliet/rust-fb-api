CREATE TABLE IF NOT EXISTS announcements (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id TEXT         NOT NULL,
    message    TEXT         NOT NULL,
    frequency  VARCHAR(50)  NOT NULL DEFAULT 'daily',
    last_posted TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_announcements_channel_id ON announcements(channel_id);
CREATE INDEX IF NOT EXISTS idx_announcements_last_posted ON announcements(last_posted);
