-- Test UPDATE with subquery
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
CREATE TABLE upd_foo (id bigint, value text);
CREATE TABLE upd_bar (id bigint, value text);
INSERT INTO upd_foo SELECT i, 'foo' || i FROM generate_series(1, 10) i;
INSERT INTO upd_bar SELECT i, 'bar' || i FROM generate_series(1, 10) i;

-- Test UPDATE with subquery
EXPLAIN (COSTS OFF) UPDATE upd_foo SET value = (SELECT value FROM upd_bar WHERE upd_bar.id = upd_foo.id);
UPDATE upd_foo SET value = (SELECT value FROM upd_bar WHERE upd_bar.id = upd_foo.id);

-- Cleanup
DROP TABLE upd_foo, upd_bar;
