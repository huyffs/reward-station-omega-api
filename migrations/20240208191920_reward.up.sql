CREATE TABLE reward (
  id uuid NOT NULL,
  issuer_id TEXT,
  category SMALLINT,
  country_id SMALLINT,
  tandc TEXT,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  images TEXT [] NOT NULL DEFAULT '{}' check (array_position(images, null) is null),
  active_from date,
  active_until date,
  valid_from date,
  valid_until date,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone,
  PRIMARY KEY (id)
);

CREATE TABLE coupon (
  reward_id uuid NOT NULL,
  number BIGINT NOT NULL,
  url TEXT NOT NULL,
  user_id TEXT,
  minted_at timestamp with time zone,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone,
  redeemed_at timestamp with time zone,
  redeemed_meta JSONB,
  PRIMARY KEY (reward_id, number)
);

CREATE TABLE project__reward (
  org_id uuid NOT NULL,
  project_id uuid NOT NULL,
  reward_id uuid NOT NULL,
  point BIGINT NOT NULL,
  multiple BOOLEAN NOT NULL DEFAULT false,
  approved BOOLEAN NOT NULL DEFAULT false,
  active BOOLEAN NOT NULL DEFAULT true,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone,
  PRIMARY KEY (project_id, reward_id),
  CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project(id),
  CONSTRAINT fk_reward FOREIGN KEY (reward_id) REFERENCES reward(id)
);
