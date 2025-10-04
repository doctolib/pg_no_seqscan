-- Test that EXPLAIN queries are ignored

SET pg_no_seqscan.level = ERROR;
CREATE TABLE test_explain AS (SELECT * FROM generate_series(1,10) AS id);

-- EXPLAIN should not trigger errors
EXPLAIN SELECT * FROM test_explain;

-- EXPLAIN ANALYZE should not trigger errors
EXPLAIN (ANALYZE, TIMING OFF, SUMMARY OFF) SELECT * FROM test_explain;

-- But regular query should trigger error
SELECT * FROM test_explain;

-- cleanup
drop table test_explain;