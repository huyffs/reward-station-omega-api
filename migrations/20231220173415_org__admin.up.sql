CREATE TABLE org__admin (
  admin_id uuid NOT NULL,
  org_id uuid NOT NULL,
  PRIMARY KEY (admin_id, org_id),
  CONSTRAINT fk_admin FOREIGN KEY (admin_id) REFERENCES admin(id),
  CONSTRAINT fk_org FOREIGN KEY (org_id) REFERENCES org(id)
);
