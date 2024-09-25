CREATE TABLE mezzofy_call_log (
  id BIGSERIAL PRIMARY KEY,
  idemp_key TEXT,
  method TEXT NOT NULL,
  url TEXT NOT NULL,
  req_headers JSONB NOT NULL DEFAULT '{}' :: JSONB,
  req_body TEXT,
  res_headers JSONB DEFAULT '{}' :: JSONB,
  res_body TEXT,
  status SMALLINT,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone
);
