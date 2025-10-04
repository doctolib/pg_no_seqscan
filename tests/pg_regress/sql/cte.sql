-- Test CTE
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
CREATE TABLE test_cte AS (SELECT * FROM generate_series(1,10) as id);

-- Show plan
EXPLAIN (COSTS OFF)
WITH cte AS (SELECT * FROM test_cte) SELECT * FROM cte;

-- Expect standard query execution as we are not querying a real table
WITH cte AS (SELECT * FROM test_cte) SELECT * FROM cte;

-- Cleanup
DROP TABLE test_cte;
