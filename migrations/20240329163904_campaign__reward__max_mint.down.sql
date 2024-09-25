ALTER TABLE campaign__reward ADD COLUMN mint_limit INTEGER;
ALTER TABLE campaign__reward ADD COLUMN multiple BOOLEAN NOT NULL DEFAULT false;
UPDATE campaign__reward SET multiple = true WHERE user_mint IS NULL OR user_mint = 0;
ALTER TABLE campaign__reward DROP COLUMN max_mint;
ALTER TABLE campaign__reward DROP COLUMN user_mint;
