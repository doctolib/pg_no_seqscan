-- Test database filtering
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;

CREATE TABLE test_db AS (SELECT * FROM generate_series(1,10) as id);

-- Blocks query execution as check_databases is empty thus all databases are checked
SET pg_no_seqscan.check_databases = '';
EXPLAIN (COSTS OFF) SELECT * FROM test_db;
SELECT * FROM test_db; -- Should error

-- Allows query execution as check_databases is not set to the current database
SET pg_no_seqscan.check_databases = 'postgres';
SELECT * FROM test_db; -- Should pass


-- Current database depends on the test runner (ex: contrib_regression) that's why we use this hack to not return the database name in the output.
SELECT ''
EXCEPT
SELECT set_config('pg_no_seqscan.check_databases', current_database(), false);

-- Blocks query execution as check_databases is not set to the current database
SELECT * FROM test_db; -- Should error

-- Cleanup
DROP TABLE test_db;
RESET pg_no_seqscan.check_databases;
