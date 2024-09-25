DROP TRIGGER notify_engage_event_insert ON engage_event;
DROP TRIGGER engage_insert ON engage;
DROP TRIGGER engage_update ON engage;

DROP FUNCTION with_campaign_relation;
DROP FUNCTION copy_campaign_relation;
DROP FUNCTION notify_engage_event;
DROP FUNCTION log_engage_insert;
DROP FUNCTION log_engage_update;

DROP TABLE engage_event CASCADE;
