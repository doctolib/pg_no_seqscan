-- Test that EXPLAIN (COSTS OFF) queries are ignored
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
CREATE TABLE test_explain AS (SELECT * FROM generate_series(1,10) AS id);

-- EXPLAIN (COSTS OFF) should not trigger errors
EXPLAIN (COSTS OFF) SELECT * FROM test_explain;

-- EXPLAIN (COSTS OFF) ANALYZE should not trigger errors
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF) SELECT * FROM test_explain;

-- But regular query should trigger error
SELECT * FROM test_explain;

-- cleanup
drop table test_explain;