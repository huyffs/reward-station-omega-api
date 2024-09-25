ALTER TABLE campaign ALTER COLUMN start_at DROP NOT NULL;
ALTER TABLE campaign ALTER COLUMN start_at TYPE date;
ALTER TABLE campaign ALTER COLUMN end_at DROP NOT NULL;
ALTER TABLE campaign ALTER COLUMN end_at TYPE date;
ALTER TABLE campaign ADD COLUMN voucher_policy SMALLINT NOT NULL DEFAULT 1;
ALTER TABLE campaign ADD COLUMN voucher_expire_at date;

CREATE TABLE voucher (
  org_id uuid NOT NULL,
  project_id uuid NOT NULL,
  campaign_id uuid NOT NULL,
  chain_id BIGINT NOT NULL,
  signer_address TEXT NOT NULL,
  user_id TEXT NOT NULL,
  task_id TEXT NOT NULL,
  value BIGINT NOT NULL,
  balance BIGINT NOT NULL,
  valid_from date,
  valid_until date,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone,
  PRIMARY KEY (campaign_id, chain_id, signer_address, task_id),
  CONSTRAINT fk_org FOREIGN KEY (org_id) REFERENCES org(id),
  CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project(id),
  CONSTRAINT fk_campaign FOREIGN KEY (campaign_id) REFERENCES campaign(id)
);
