CREATE TABLE registrations (
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
    approved BOOL NULL,
    CONSTRAINT registrations_pkey PRIMARY KEY (id),
    INDEX idx_applicantid (applicantid)
);
