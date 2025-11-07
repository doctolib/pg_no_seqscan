-- Test that indexed queries don't trigger errors
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
SET enable_seqscan = off;
CREATE TABLE test_pk (id bigint PRIMARY KEY);
INSERT INTO test_pk SELECT generate_series(1,10);

-- Allows query execution as an index is used
EXPLAIN (COSTS OFF)
SELECT * FROM test_pk WHERE id=1;
SELECT * FROM test_pk WHERE id=1;

-- Cleanup
DROP TABLE test_pk;
