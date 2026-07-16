CREATE TABLE birthdays (
    day INT4 NOT NULL,
    month INT4 NOT NULL,
    name VARCHAR(255) NOT NULL,
    snowflake VARCHAR(255) NOT NULL,
    CONSTRAINT birthdays_pk PRIMARY KEY (snowflake),
    UNIQUE INDEX snowflake_un (snowflake)
);
