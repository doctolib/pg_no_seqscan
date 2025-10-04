-- Test ignoring seqscans with skip comments

SET pg_no_seqscan.level = ERROR;
CREATE TABLE test_skip AS (SELECT * FROM generate_series(1,10) AS id);

EXPLAIN SELECT * FROM test_skip;
SELECT * FROM test_skip; -- should fail

-- Test with skip comment variations
SELECT * FROM test_skip /* pg_no_seqscan_skip */;
SELECT * FROM test_skip /* host_name:a-b-1.2.foo,db:my_database,git:0123456789abcdef,pg_no_seqscan_skip,path:/foo/source.java:108`(<>)' */;
SELECT * FROM test_skip /*pg_no_seqscan_skip*/;

DROP TABLE test_skip;
