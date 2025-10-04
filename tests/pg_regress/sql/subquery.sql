-- Test subquery detection
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
CREATE TABLE test_subq AS (SELECT * FROM generate_series(1,10) as id);

-- Test subquery
SELECT * FROM (SELECT * FROM test_subq) as subq;

-- Cleanup
DROP TABLE test_subq;