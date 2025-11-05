-- Test that indexed queries don't trigger errors
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
SET enable_seqscan = off;
CREATE TABLE test_pk (id bigint PRIMARY KEY);
INSERT INTO test_pk SELECT generate_series(1,10);


-- Query by primary key should not error
EXPLAIN (COSTS OFF)
SELECT * FROM test_pk WHERE id=1;
SELECT * FROM test_pk WHERE id=1;

-- Cleanup
DROP TABLE test_pk;
