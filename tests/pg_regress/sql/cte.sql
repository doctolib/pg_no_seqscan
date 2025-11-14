-- Test detection in CTE
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
CREATE TABLE test_cte AS (SELECT * FROM generate_series(1,10) as id);

-- Blocks query execution as seqscan is done in the CTE
EXPLAIN (COSTS OFF)
WITH cte AS MATERIALIZED (SELECT * FROM test_cte LIMIT 10) SELECT * FROM cte ORDER BY id;

WITH cte AS MATERIALIZED (SELECT * FROM test_cte LIMIT 10) SELECT * FROM cte;

-- Cleanup
DROP TABLE test_cte;
