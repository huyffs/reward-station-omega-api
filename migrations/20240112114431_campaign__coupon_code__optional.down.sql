UPDATE campaign SET coupon_code = '' WHERE coupon_code IS NULL;
ALTER TABLE campaign ALTER COLUMN coupon_code SET NOT NULL;
