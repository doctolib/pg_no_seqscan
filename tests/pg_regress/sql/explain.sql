-- Test that EXPLAIN (COSTS OFF) queries are ignored
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;

CREATE TABLE test_explain AS (SELECT * FROM generate_series(1,10) AS id);

-- Allows query execution as it is an EXPLAIN, and no seqscan will occur
EXPLAIN (COSTS OFF)
SELECT * FROM test_explain;

-- Allows query execution as it is an EXPLAIN ANALYZE, as it may be useful to understand the plan
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF) SELECT * FROM test_explain;

-- Allows query execution as it is an EXPLAIN ANALYZE, even for parallel queries
SET force_parallel_mode = on;
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF) select count(*) from test_explain;
SET force_parallel_mode = off;

-- Allows query execution as it is an EXPLAIN ANALYZE, even for parallel queries
SET parallel_setup_cost = 0;
SET parallel_tuple_cost = 0.000001;
CREATE TABLE test_explain_parallel (id, migrated_at) AS (SELECT generate_series(1, 800000)::bigint id, null::timestamp without time zone);
EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF) select count(*) from test_explain_parallel;
SET parallel_setup_cost TO DEFAULT;
SET parallel_tuple_cost TO DEFAULT;

-- Blocks query execution as this is the real query
SELECT * FROM test_explain;


-- cleanup
drop table test_explain;
drop table test_explain_parallel;

