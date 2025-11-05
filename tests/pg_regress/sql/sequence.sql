-- Test detection in SEQUENCE
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
CREATE SEQUENCE test_seq;
-- Show plan:
EXPLAIN (COSTS OFF)
SELECT last_value FROM test_seq;
-- Querying a sequence should not cause error
SELECT last_value FROM test_seq;

-- cleanup
DROP SEQUENCE test_seq;
