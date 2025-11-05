-- Test detection in BITMAP AND
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
SET enable_seqscan = off;
CREATE TABLE foo (id bigint, value text, category text);
INSERT INTO foo SELECT i, 'value' || i, CASE WHEN i % 2 = 0 THEN 'even' ELSE 'odd' END FROM generate_series(1, 10000) i;
CREATE INDEX idx_foo_value ON foo(value);
CREATE INDEX idx_foo_category ON foo(category);

-- Show the plan
EXPLAIN (COSTS OFF)
SELECT count(*) FROM foo WHERE value = 'value1' AND category = 'even';

-- Expect standard query execution, as it uses 'BITMAP AND'
SELECT count(*) FROM foo WHERE value = 'value1' AND category = 'even';

-- Cleanup
DROP TABLE foo;
