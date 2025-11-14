-- Test that EXPLAIN (COSTS OFF) queries are ignored
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;

CREATE TABLE test_explain AS (SELECT * FROM generate_series(1,10) AS id);

-- Allows query execution as it is an EXPLAIN, and no seqscan will occur
EXPLAIN (COSTS OFF)
SELECT * FROM test_explain;

-- Allows query execution as it is an EXPLAIN ANALYZE, as it may be useful to understand the plan
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF) SELECT * FROM test_explain;

-- Blocks query execution as this is the real query
SELECT * FROM test_explain;

-- cleanup
drop table test_explain;
