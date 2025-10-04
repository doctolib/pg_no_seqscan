-- Test that indexed queries don't trigger errors

SET pg_no_seqscan.level = ERROR;
CREATE TABLE test_pk (id bigint PRIMARY KEY);
INSERT INTO test_pk SELECT generate_series(1,10);


-- Query by primary key should not error
EXPLAIN SELECT * FROM test_pk WHERE id=1;
SELECT * FROM test_pk WHERE id=1;

DROP TABLE test_pk;