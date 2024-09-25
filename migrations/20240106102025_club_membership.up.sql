--
-- Functions
--
--
-- Subscribe to a project on participation insert
--
CREATE FUNCTION subscribe_club() RETURNS trigger AS $$ DECLARE subscribed_time timestamp with time zone;

num_res BIGINT;

BEGIN
  SELECT
    subscribed_at,
    count(subscribed_at)
  FROM
    club_membership
  WHERE
    project_id = NEW.project_id
    AND chain_id = NEW.chain_id
    AND signer_address = NEW.signer_address GROUP BY subscribed_at INTO subscribed_time,
    num_res;

  IF num_res IS NULL THEN
    INSERT INTO
      club_membership(
        project_id,
        chain_id,
        signer_address,
        subscribed_at
      )
    VALUES
      (
        NEW.project_id,
        NEW.chain_id,
        NEW.signer_address,
        NEW.created_at
      );
  ELSEIF subscribed_time IS NULL THEN
    UPDATE
      club_membership
    SET
      subscribed_at = NEW.created_at
    WHERE
      project_id = NEW.project_id
      AND chain_id = NEW.chain_id
      AND signer_address = NEW.signer_address;
  END IF;

  RETURN NEW;
END;

$$ LANGUAGE plpgsql;
--
-- Join to a project when completed first campaign
--
CREATE FUNCTION join_club() RETURNS trigger AS $$ DECLARE joined_time timestamp with time zone;

num_res BIGINT;

BEGIN
	IF NEW.coupon_serial is distinct from OLD.coupon_serial THEN
    UPDATE
      club_membership
    SET
      joined_at = NEW.created_at
    WHERE
      project_id = NEW.project_id
      AND chain_id = NEW.chain_id
      AND signer_address = NEW.signer_address
      AND joined_at IS NULL;
  END IF;

  RETURN NEW;
END;

$$ LANGUAGE plpgsql;

--
-- Tables
--
CREATE TABLE club_membership (
  project_id uuid NOT NULL,
  chain_id BIGINT NOT NULL,
  signer_address TEXT NOT NULL,
  subscribed_at timestamp with time zone,
  joined_at timestamp with time zone,
  PRIMARY KEY (project_id, chain_id, signer_address),
  CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project(id)
);

--
-- Triggers
--
CREATE TRIGGER subscribe_club_on_engage_insert
AFTER
INSERT
  ON engage FOR EACH ROW EXECUTE PROCEDURE subscribe_club();

CREATE TRIGGER join_club_on_engage_update
AFTER
UPDATE
  ON engage FOR EACH ROW EXECUTE PROCEDURE join_club();
