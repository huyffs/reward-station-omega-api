CREATE TABLE project__member (
  member_id uuid NOT NULL,
  project_id uuid NOT NULL,
  point BIGINT NOT NULL DEFAULT 0,
  PRIMARY KEY (member_id, project_id),
  CONSTRAINT fk_member FOREIGN KEY (member_id) REFERENCES member(id),
  CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project(id)
);
