CREATE TABLE campaign__reward (
  org_id uuid NOT NULL,
  project_id uuid NOT NULL,
  campaign_id uuid NOT NULL,
  reward_id uuid NOT NULL,
  point BIGINT NOT NULL,
  multiple BOOLEAN NOT NULL DEFAULT false,
  approved BOOLEAN NOT NULL DEFAULT false,
  active BOOLEAN NOT NULL DEFAULT true,
  mint_limit INTEGER,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone,
  PRIMARY KEY (campaign_id, reward_id),
  CONSTRAINT fk_campaign FOREIGN KEY (campaign_id) REFERENCES campaign(id),
  CONSTRAINT fk_reward FOREIGN KEY (reward_id) REFERENCES reward(id)
);
