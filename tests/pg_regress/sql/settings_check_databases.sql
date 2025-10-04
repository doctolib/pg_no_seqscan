-- Test database filtering
SET pg_no_seqscan.level = ERROR;
CREATE TABLE test_db AS (SELECT * FROM generate_series(1,10) as id);
EXPLAIN SELECT * FROM test_db;

-- Empty check_databases should check all databases
SET pg_no_seqscan.check_databases = '';
SELECT * FROM test_db; -- Should error

-- Non-matching database should be ignored
-- Note: regress tests run in database named 'contrib_regression' or similar
SET pg_no_seqscan.check_databases = 'postgres';
SELECT * FROM test_db; -- Should pass

-- Matching database should be checked
-- Note: pg_regress uses 'pg_no_seqscan_regress' as database name
SET pg_no_seqscan.check_databases = 'pg_no_seqscan_regress';
SELECT * FROM test_db; -- Should error

-- Cleanup
DROP TABLE test_db;
