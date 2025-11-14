-- Test user filtering
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
CREATE TABLE test_user_table (id serial);

CREATE USER test_user;
SET pg_no_seqscan.ignore_users = 'test_user_2,test_user';

-- Blocks query execution as current user is not ignored
EXPLAIN (COSTS OFF) SELECT * FROM test_user_table;
SELECT * FROM test_user_table;

GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE test_user_table TO test_user;
SET SESSION AUTHORIZATION test_user;

-- Allows query execution as current user is ignored
SELECT * FROM test_user_table;

-- Reset session
RESET SESSION AUTHORIZATION;
RESET pg_no_seqscan.ignore_users;
DROP TABLE test_user_table;
DROP USER test_user;
