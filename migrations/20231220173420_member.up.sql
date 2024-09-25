CREATE TABLE member (
  id uuid NOT NULL DEFAULT gen_random_uuid (),
  wallets JSONB DEFAULT '{}' :: jsonb NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone,
  PRIMARY KEY (id)
);
