-- Test detection in APPEND
SELECT 'Before PG 18?', setting::int / 10000 < 18 from pg_settings where name = 'server_version_num' ; -- output changed slightly from PG18
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
SET enable_seqscan = off;

CREATE TABLE test_for_append (id int, data text);
INSERT INTO test_for_append SELECT i, 'data' || i FROM generate_series(1, 5) i;

-- Blocks query execution as both parts are seq scans
EXPLAIN (COSTS OFF) (SELECT * FROM test_for_append ORDER BY id LIMIT 1) UNION ALL (SELECT * FROM test_for_append ORDER BY id desc LIMIT 1);

(SELECT * FROM test_for_append ORDER BY id LIMIT 1) UNION ALL (SELECT * FROM test_for_append ORDER BY id desc LIMIT 1);

-- Blocks query execution as one part of the append is a seq scan.
CREATE INDEX test_for_append_idx ON test_for_append (id);
EXPLAIN (COSTS OFF) (SELECT * FROM test_for_append ORDER BY id LIMIT 1) UNION ALL (SELECT * FROM test_for_append ORDER BY random() LIMIT 1);
(SELECT * FROM test_for_append ORDER BY id LIMIT 1) UNION ALL (SELECT * FROM test_for_append ORDER BY random() LIMIT 1);

-- Allows query execution as both parts are index scans
EXPLAIN (COSTS OFF) (SELECT * FROM test_for_append ORDER BY id LIMIT 1) UNION ALL (SELECT * FROM test_for_append ORDER BY id LIMIT 1 OFFSET 2);
(SELECT * FROM test_for_append ORDER BY id LIMIT 1) UNION ALL (SELECT * FROM test_for_append ORDER BY id LIMIT 1 OFFSET 2);

DROP TABLE test_for_append;
