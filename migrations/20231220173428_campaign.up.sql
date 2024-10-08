CREATE TABLE campaign (
  org_id uuid NOT NULL,
  project_id uuid NOT NULL,
  id uuid NOT NULL DEFAULT gen_random_uuid (),
  name TEXT NOT NULL,
  description TEXT,
  logo TEXT,
  images TEXT [] NOT NULL DEFAULT '{}' check (array_position(images, null) is null),
  coupon_code TEXT NOT NULL,
  budget numeric,
  chain_id BIGINT NOT NULL,
  contract_address TEXT NOT NULL,
  condition_info TEXT,
  reward_amount numeric,
  reward_info TEXT,
  tasks JSONB DEFAULT '[]' :: jsonb NOT NULL,
  start_at timestamp with time zone NOT NULL DEFAULT now(),
  end_at timestamp with time zone,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone,
  PRIMARY KEY (id),
  CONSTRAINT fk_org FOREIGN KEY (org_id) REFERENCES org(id),
  CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project(id)
);
