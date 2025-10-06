-- Test basic seqscan detection at different levels
-- Setup
LOAD 'pg_no_seqscan';
SET client_min_messages = NOTICE;
CREATE TABLE basic_seqscan AS (SELECT * FROM generate_series(1,10) AS id);
EXPLAIN (COSTS OFF) SELECT * FROM basic_seqscan;

-- Test 1: Level OFF should ignore seqscans
SET pg_no_seqscan.level = OFF;
SELECT * FROM basic_seqscan;

-- Test 2: Level WARN should warn on seqscans
SET pg_no_seqscan.level = WARN;
SELECT * FROM basic_seqscan;

-- Test 3: Level ERROR should error on seqscans
SET pg_no_seqscan.level = ERROR;
SELECT * FROM basic_seqscan; -- This should fail

-- Cleanup
DROP TABLE basic_seqscan;
RESET client_min_messages;
