-- Test CTE
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
CREATE TABLE test_cte AS (SELECT * FROM generate_series(1,10) as id);

-- Show plan
EXPLAIN (COSTS OFF)
WITH cte AS MATERIALIZED (SELECT * FROM test_cte LIMIT 10) SELECT * FROM cte ORDER BY id;

-- Expect to fail
WITH cte AS MATERIALIZED (SELECT * FROM test_cte LIMIT 10) SELECT * FROM cte;

-- Cleanup
DROP TABLE test_cte;
