ALTER TABLE engage ADD COLUMN user_name TEXT;
ALTER TABLE engage_event ADD COLUMN user_name TEXT;

CREATE OR REPLACE FUNCTION log_engage_insert() RETURNS TRIGGER LANGUAGE PLPGSQL AS $$ BEGIN
INSERT INTO
  engage_event (
    org_id,
    project_id,
    campaign_id,
    chain_id,
    signer_address,
    user_id,
    user_name,
    new_submissions,
    new_accepted,
    new_coupon_serial,
    new_coupon_url,
    created_at
  )
VALUES
(
    NEW.org_id,
    NEW.project_id,
    NEW.campaign_id,
    NEW.chain_id,
    NEW.signer_address,
    NEW.user_id,
    NEW.user_name,
    NEW.submissions,
    NEW.accepted,
    NEW.coupon_serial,
    NEW.coupon_url,
    now()
  );

RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION log_engage_update() RETURNS TRIGGER LANGUAGE PLPGSQL AS $$ BEGIN
INSERT INTO
  engage_event (
    org_id,
    project_id,
    campaign_id,
    chain_id,
    signer_address,
    user_id,
    old_submissions,
    old_accepted,
    old_coupon_serial,
    old_coupon_url,
    new_submissions,
    new_accepted,
    new_coupon_serial,
    new_coupon_url,
    created_at
  )
VALUES
(
    NEW.org_id,
    NEW.project_id,
    NEW.campaign_id,
    NEW.chain_id,
    NEW.signer_address,
    NEW.user_id,
    OLD.submissions,
    OLD.accepted,
    OLD.coupon_serial,
    OLD.coupon_url,
    NEW.submissions,
    NEW.accepted,
    NEW.coupon_serial,
    NEW.coupon_url,
    now()
  );

RETURN NEW;
END;
$$;

