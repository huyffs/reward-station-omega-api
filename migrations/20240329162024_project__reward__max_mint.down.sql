ALTER TABLE project__reward ADD COLUMN multiple BOOLEAN NOT NULL DEFAULT false;
UPDATE project__reward SET multiple = true WHERE user_mint IS NULL OR user_mint = 0;
ALTER TABLE project__reward DROP COLUMN max_mint;
ALTER TABLE project__reward DROP COLUMN user_mint;
