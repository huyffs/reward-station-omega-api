ALTER TABLE project__reward ADD COLUMN max_mint BIGINT;
ALTER TABLE project__reward ADD COLUMN user_mint BIGINT;
UPDATE project__reward SET user_mint = 1 WHERE multiple = false;
ALTER TABLE project__reward DROP COLUMN multiple;
