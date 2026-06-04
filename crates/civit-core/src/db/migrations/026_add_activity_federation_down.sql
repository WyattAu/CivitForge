-- Down migration 026: Drop activity feed + federation tables

DROP TABLE IF EXISTS federation_activities;
DROP TABLE IF EXISTS federation_actors;
DROP TABLE IF EXISTS activity_events;
