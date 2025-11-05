-- Test basic seqscan detection at different levels
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
SET client_min_messages = NOTICE;
CREATE TABLE basic_seqscan AS (SELECT * FROM generate_series(1,10) AS id);
EXPLAIN (COSTS OFF) SELECT * FROM basic_seqscan;

-- Level OFF should ignore seqscans
SET pg_no_seqscan.level = OFF;
SELECT * FROM basic_seqscan;

-- Level WARN should warn on seqscans
SET pg_no_seqscan.level = WARN;
SELECT * FROM basic_seqscan;

-- Level ERROR should error on seqscans
SET pg_no_seqscan.level = ERROR;
SELECT * FROM basic_seqscan; -- This should fail

-- Cleanup
DROP TABLE basic_seqscan;
RESET client_min_messages;
SET pg_no_seqscan.level = ERROR;
