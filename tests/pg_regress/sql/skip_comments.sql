-- Test ignoring seqscans with skip comments
CREATE TABLE test_skip AS (SELECT * FROM generate_series(1,10) AS id);

-- Show the plan
EXPLAIN (COSTS OFF)
SELECT * FROM test_skip;
-- This query should fail:
SELECT * FROM test_skip;

-- Test with skip comment variations
SELECT * FROM test_skip /* pg_no_seqscan_skip */;
SELECT * FROM test_skip /* host_name:a-b-1.2.foo,db:my_database,git:0123456789abcdef,pg_no_seqscan_skip,path:/foo/source.java:108`(<>)' */;
SELECT * FROM test_skip /*pg_no_seqscan_skip*/;

-- Cleanup
DROP TABLE test_skip;
