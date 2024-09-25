CREATE TABLE admin (
  id uuid NOT NULL DEFAULT gen_random_uuid (),
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone,
  PRIMARY KEY (id)
);
