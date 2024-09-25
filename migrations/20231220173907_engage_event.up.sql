CREATE TABLE engage_event (
  id BIGSERIAL PRIMARY KEY,
  org_id uuid NOT NULL,
  project_id uuid NOT NULL,
  campaign_id uuid NOT NULL,
  chain_id BIGINT NOT NULL,
  signer_address TEXT NOT NULL,
  user_id TEXT NOT NULL,
  old_submissions JSONB,
  old_accepted JSONB,
  old_coupon_serial TEXT,
  old_coupon_url TEXT,
  new_submissions JSONB,
  new_accepted JSONB,
  new_coupon_serial TEXT,
  new_coupon_url TEXT,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  CONSTRAINT fk_org FOREIGN KEY (org_id) REFERENCES org(id),
  CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project(id),
  CONSTRAINT fk_campaign FOREIGN KEY (campaign_id) REFERENCES campaign(id)
);

---
--- Insert new engage_event on engage insert
---
CREATE FUNCTION log_engage_insert() RETURNS TRIGGER LANGUAGE PLPGSQL AS $$ BEGIN
INSERT INTO
  engage_event (
    org_id,
    project_id,
    campaign_id,
    chain_id,
    signer_address,
    user_id,
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
    NEW.submissions,
    NEW.accepted,
    NEW.coupon_serial,
    NEW.coupon_url,
    now()
  );

RETURN NEW;
END;
$$;

--
--- Insert new engage_event on engage update
--
CREATE FUNCTION log_engage_update() RETURNS TRIGGER LANGUAGE PLPGSQL AS $$ BEGIN
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

--
-- Set org & project ID in new engage records
--
CREATE FUNCTION with_campaign_relation(_tbl regclass) RETURNS VOID AS
$$ BEGIN

EXECUTE format(
  'CREATE TRIGGER with_campaign_relation BEFORE INSERT ON %s
    FOR EACH ROW EXECUTE PROCEDURE copy_campaign_relation()',
  _tbl
);

END;
$$ LANGUAGE plpgsql;

--
-- Function to get org_id and project_id from campaign table
--
CREATE FUNCTION copy_campaign_relation() RETURNS trigger AS
$$ DECLARE

org_id uuid;
proj_id uuid;

BEGIN
SELECT
  org_id,
  project_id
FROM campaign
WHERE
  id = NEW.campaign_id INTO org_id,
  proj_id;

NEW.org_id := org_id;
NEW.project_id := proj_id;

RETURN NEW;

END;
$$ LANGUAGE plpgsql;

--
-- Function to notify engage event
--
CREATE FUNCTION notify_engage_event() RETURNS trigger AS
$$ BEGIN

PERFORM pg_notify('engage', row_to_json(NEW) :: text);
RETURN NEW;

END;
$$ LANGUAGE plpgsql;

--
-- Notify on new engage_event
--
CREATE TRIGGER notify_engage_event_insert
AFTER INSERT ON engage_event
  FOR EACH ROW EXECUTE PROCEDURE notify_engage_event();

---
--- Create a log in engage_event on new engage insert
---
CREATE TRIGGER engage_insert
AFTER INSERT ON engage
  FOR EACH ROW EXECUTE PROCEDURE log_engage_insert();

--
-- Create a log in engage engage_event
--
CREATE TRIGGER engage_update
AFTER UPDATE OF
  submissions,
  accepted,
  coupon_serial,
  coupon_url
ON engage
  FOR EACH ROW EXECUTE PROCEDURE log_engage_update();
