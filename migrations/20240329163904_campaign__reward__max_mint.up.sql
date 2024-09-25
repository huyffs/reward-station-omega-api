ALTER TABLE campaign__reward ADD COLUMN max_mint BIGINT;
ALTER TABLE campaign__reward ADD COLUMN user_mint BIGINT;
UPDATE campaign__reward SET user_mint = 1 WHERE multiple = false;
ALTER TABLE campaign__reward DROP COLUMN multiple;
ALTER TABLE campaign__reward DROP COLUMN mint_limit;
