-- Test CTE detection

SET pg_no_seqscan.level = ERROR;
CREATE TABLE test_cte AS (SELECT * FROM generate_series(1,10) as id);

-- Test CTE
EXPLAIN WITH cte AS (SELECT * FROM test_cte) SELECT * FROM cte;

WITH cte AS (SELECT * FROM test_cte) SELECT * FROM cte;

-- Cleanup
DROP TABLE test_cte;