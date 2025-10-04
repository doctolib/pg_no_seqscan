-- Test that indexed queries don't trigger errors

SET pg_no_seqscan.level = ERROR;

-- Querying a sequence should not error
CREATE SEQUENCE test_seq;
EXPLAIN SELECT last_value FROM test_seq;
SELECT last_value FROM test_seq;

-- cleanup
DROP SEQUENCE test_seq;
