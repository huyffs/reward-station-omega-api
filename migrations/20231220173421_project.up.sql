CREATE TABLE project (
  org_id uuid NOT NULL,
  id uuid NOT NULL DEFAULT gen_random_uuid (),
  name TEXT NOT NULL,
  description TEXT,
  logo TEXT,
  images TEXT [] NOT NULL DEFAULT '{}' check (array_position(images, null) is null),
  website TEXT,
  networks JSONB DEFAULT '{}' :: jsonb NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone,
  PRIMARY KEY (id),
  CONSTRAINT fk_org FOREIGN KEY (org_id) REFERENCES org(id)
);
