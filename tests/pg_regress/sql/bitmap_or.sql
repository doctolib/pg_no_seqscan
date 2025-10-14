-- Test detection in bitmap or
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;

-- Setup
CREATE TABLE foo (id bigint, value text, category text);
INSERT INTO foo SELECT i, 'value' || i, CASE WHEN i % 2 = 0 THEN 'even' ELSE 'odd' END FROM generate_series(1, 600) i;
CREATE INDEX idx_foo_value ON foo(value);
CREATE INDEX idx_foo_category ON foo(category);

-- Show query plan
EXPLAIN SELECT count(*) FROM foo WHERE value = 'value1' OR category = 'even';

-- Should pass as it uses bitmap or
SELECT count(*) FROM foo WHERE value = 'value1' OR category = 'even';

-- Cleanup
DROP TABLE foo;
