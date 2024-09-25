DROP TABLE voucher CASCADE;
ALTER TABLE campaign DROP COLUMN voucher_expire_at;
ALTER TABLE campaign DROP COLUMN voucher_policy;
ALTER TABLE campaign ALTER COLUMN end_at TYPE timestamp with time zone;
ALTER TABLE campaign ALTER COLUMN end_at SET NOT NULL;
ALTER TABLE campaign ALTER COLUMN start_at TYPE timestamp with time zone;
ALTER TABLE campaign ALTER COLUMN start_at SET NOT NULL;
