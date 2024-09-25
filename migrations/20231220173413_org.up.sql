CREATE TABLE org (
  id uuid NOT NULL DEFAULT gen_random_uuid (),
  name TEXT NOT NULL,
  logo TEXT,
  admins JSONB NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone,
  PRIMARY KEY (id)
);
