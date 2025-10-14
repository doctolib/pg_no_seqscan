-- Test detection in bitmap or
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;

-- Setup
CREATE TABLE foo (id bigint, value text, category text);
INSERT INTO foo SELECT i, 'value' || i, CASE WHEN i % 2 = 0 THEN 'even' ELSE 'odd' END FROM generate_series(1, 10000) i;
CREATE INDEX idx_foo_value ON foo(value);
CREATE INDEX idx_foo_category ON foo(category);

-- Show query plan
EXPLAIN SELECT count(*) FROM foo WHERE value = 'value1' AND category = 'even';

-- Should pass as it uses bitmap and
SELECT count(*) FROM foo WHERE value LIKE 'value%' AND category = 'even';

-- Cleanup
DROP TABLE foo;
