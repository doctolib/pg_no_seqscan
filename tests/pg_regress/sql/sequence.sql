-- Test detection in SEQUENCE
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
CREATE SEQUENCE test_seq;

-- Allows query execution as it's allowed to seqscan a sequence
EXPLAIN (COSTS OFF)
SELECT last_value FROM test_seq;
SELECT last_value FROM test_seq;

-- cleanup
DROP SEQUENCE test_seq;
