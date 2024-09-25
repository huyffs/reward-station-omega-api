CREATE TABLE engage (
  org_id uuid NOT NULL,
  project_id uuid NOT NULL,
  campaign_id uuid NOT NULL,
  chain_id BIGINT NOT NULL,
  signer_address TEXT NOT NULL,
  user_id TEXT NOT NULL,
  submissions JSONB DEFAULT '{}' :: jsonb NOT NULL,
  accepted JSONB DEFAULT '{}' :: jsonb NOT NULL,
  messages JSONB DEFAULT '{}' :: jsonb NOT NULL,
  coupon_issue_id TEXT,
  coupon_serial TEXT,
  coupon_url TEXT,
  country_id SMALLINT,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone,
  PRIMARY KEY (campaign_id, chain_id, signer_address),
  CONSTRAINT fk_campaign FOREIGN KEY (campaign_id) REFERENCES campaign(id)
);
