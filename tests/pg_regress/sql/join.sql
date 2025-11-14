-- Test detection in JOIN
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
SET enable_seqscan = off;
CREATE TABLE complex_query_foo AS (SELECT * FROM generate_series(1,10) as id);
CREATE TABLE complex_query_bar AS (SELECT * FROM generate_series(1,10) as id);

-- Blocks query execution as seqscan is done on both tables
EXPLAIN (COSTS OFF)
SELECT * FROM complex_query_foo JOIN complex_query_bar ON complex_query_foo.id = complex_query_bar.id;

SELECT * FROM complex_query_foo JOIN complex_query_bar ON complex_query_foo.id = complex_query_bar.id;

create index foo_idx on complex_query_foo(id);

-- Blocks query execution as seqscan is done on bar in one part of the join
EXPLAIN (COSTS OFF)
SELECT * FROM complex_query_foo JOIN complex_query_bar ON complex_query_foo.id = complex_query_bar.id;

SELECT * FROM complex_query_foo JOIN complex_query_bar ON complex_query_foo.id = complex_query_bar.id;

-- Cleanup
DROP TABLE complex_query_foo, complex_query_bar;
