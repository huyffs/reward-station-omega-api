
CREATE TABLE project__user (
  project_id UUID NOT NULL,
  user_id TEXT NOT NULL,
  subscribed BOOLEAN,
  PRIMARY KEY (project_id, user_id),
  CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project(id)
);

--
-- Join to a project when submitted proof
--
CREATE FUNCTION subscribe_project() RETURNS trigger AS $$

BEGIN
  INSERT INTO project__user(
    project_id,
    user_id,
    subscribed
  ) VALUES (
    NEW.project_id,
    NEW.user_id,
    true
  );

  RETURN NEW;
END;

$$ LANGUAGE plpgsql;

--
-- Triggers
--
CREATE TRIGGER subscribe_project_on_engage_insert
AFTER
INSERT
  ON engage FOR EACH ROW EXECUTE PROCEDURE subscribe_project();
