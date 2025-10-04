-- Test view detection

SET pg_no_seqscan.level = ERROR;
CREATE TABLE test_view_table AS (SELECT * FROM generate_series(1,10) as id);
CREATE VIEW test_view AS SELECT * FROM test_view_table;

-- Test querying view
EXPLAIN SELECT * FROM test_view;
SELECT * FROM test_view;
