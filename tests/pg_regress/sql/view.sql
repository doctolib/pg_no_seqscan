-- Test view detection
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
CREATE TABLE test_view_table AS (SELECT * FROM generate_series(1,10) as id);
CREATE VIEW test_view AS SELECT * FROM test_view_table;

-- Test querying view
EXPLAIN (COSTS OFF)
SELECT * FROM test_view;
SELECT * FROM test_view;

-- Cleanup
DROP VIEW test_view;
DROP TABLE test_view_table;
