-- change completed_at, added in 006, to be timestamptz
SET timezone = 'UTC';  -- make sure the time zone is set properly
ALTER TABLE games ALTER completed_at TYPE TIMESTAMPTZ;
