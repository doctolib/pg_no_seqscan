-- Test detection in JOIN
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
CREATE TABLE complex_query_foo AS (SELECT * FROM generate_series(1,10) as id);
CREATE TABLE complex_query_bar AS (SELECT * FROM generate_series(1,10) as id);

-- Test JOIN
EXPLAIN (COSTS OFF)
SELECT * FROM complex_query_foo JOIN complex_query_bar ON complex_query_foo.id = complex_query_bar.id;

SELECT * FROM complex_query_foo JOIN complex_query_bar ON complex_query_foo.id = complex_query_bar.id;

-- Cleanup
DROP TABLE complex_query_foo, complex_query_bar;
