-- Test subquery detection

SET pg_no_seqscan.level = ERROR;
CREATE TABLE test_subq AS (SELECT * FROM generate_series(1,10) as id);

-- Test subquery
SELECT * FROM (SELECT * FROM test_subq) as subq;
