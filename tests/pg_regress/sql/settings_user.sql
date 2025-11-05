-- Test user filtering
CREATE TABLE test_user_table (id serial);

CREATE USER test_user;
SET pg_no_seqscan.ignore_users = 'test_user_2,test_user';

GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE test_user_table TO test_user;
SET SESSION AUTHORIZATION test_user;

-- Should not error due to the user being in ignore_users
SELECT * FROM test_user_table;

-- Reset session
RESET SESSION AUTHORIZATION;
RESET pg_no_seqscan.ignore_users;
DROP TABLE test_user_table;
DROP USER test_user;
