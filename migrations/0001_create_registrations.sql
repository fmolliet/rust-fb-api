CREATE TABLE registrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    applicant_id TEXT NOT NULL,
    curator_id TEXT,
    birthday TEXT,
    specie TEXT,
    source TEXT,
    message_id TEXT,
    finished BOOLEAN DEFAULT FALSE,
    approved BOOLEAN,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE UNIQUE INDEX uq_open_registration ON registrations(applicant_id) WHERE finished = FALSE;
CREATE INDEX idx_reg_applicant ON registrations(applicant_id);
CREATE INDEX idx_reg_finished ON registrations(finished);
