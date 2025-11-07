-- Test table filtering with ignore_tables and check_tables
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
CREATE TABLE foo (id serial);
CREATE TABLE bar (id serial);

EXPLAIN (COSTS OFF) SELECT * FROM foo;
EXPLAIN (COSTS OFF) SELECT * FROM bar;

SET pg_no_seqscan.ignore_tables = 'something,foo';
-- Allows query execution as foo is in ignore_tables setting
SELECT * FROM foo;
-- Blocks query execution as bar is not in ignore_tables setting
SELECT * FROM bar;

-- Testing now check_tables
RESET pg_no_seqscan.ignore_tables;
SET pg_no_seqscan.check_tables = 'something,foo';
-- Blocks query execution as foo is in check_tables setting
SELECT * FROM foo;
-- Allows query execution as bar is not in check_tables setting
SELECT * FROM bar;

-- Cleanup
DROP TABLE foo, bar;
RESET pg_no_seqscan.check_tables;
