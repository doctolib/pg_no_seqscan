-- Test table filtering with ignore_tables and check_tables

SET pg_no_seqscan.level = ERROR;

CREATE TABLE foo (id serial);
CREATE TABLE bar (id serial);
CREATE TABLE baz (id serial);

EXPLAIN SELECT * FROM foo;
EXPLAIN SELECT * FROM bar;
EXPLAIN SELECT * FROM baz;


-- Test ignore_tables
SET pg_no_seqscan.ignore_tables = 'something,foo,baz';
-- Only bar should error
SELECT * FROM foo;
SELECT * FROM baz;
SELECT * FROM bar;

-- Reset for next test
RESET pg_no_seqscan.ignore_tables;

-- Test check_tables
SET pg_no_seqscan.check_tables = 'something,foo,baz';
-- Error expected on foo and baz only
SELECT * FROM foo;
SELECT * FROM bar;
SELECT * FROM baz;

-- Cleanup
DROP TABLE foo, bar, baz;
RESET pg_no_seqscan.check_tables;