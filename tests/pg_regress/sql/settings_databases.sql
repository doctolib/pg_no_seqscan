-- Test database filtering
-- Setup
CREATE TABLE test_db AS (SELECT * FROM generate_series(1,10) as id);

-- Show the plan
EXPLAIN (COSTS OFF) SELECT * FROM test_db;

-- Empty check_databases should check all databases
SET pg_no_seqscan.check_databases = '';
SELECT * FROM test_db; -- Should error

-- Non-matching database should be ignored
-- Note: regress tests run in database named 'contrib_regression' or similar
SET pg_no_seqscan.check_databases = 'postgres';
SELECT * FROM test_db; -- Should pass

-- Matching database should be checked
-- Note: current database depends on the test runner, that's why we use this hack to not return it.
SELECT ''
EXCEPT
SELECT set_config('pg_no_seqscan.check_databases', current_database(), false);

SELECT * FROM test_db; -- Should error

-- Cleanup
DROP TABLE test_db;
RESET pg_no_seqscan.check_databases;
