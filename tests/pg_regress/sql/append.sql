-- Test detection in APPEND
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
SET enable_seqscan = off;
CREATE TABLE test_for_append (id int, data text);

INSERT INTO test_for_append SELECT i, 'data' || i FROM generate_series(1, 5) i;

EXPLAIN (COSTS OFF) (SELECT * FROM test_for_append ORDER BY id LIMIT 1) UNION ALL (SELECT * FROM test_for_append ORDER BY id desc LIMIT 1);

(SELECT * FROM test_for_append ORDER BY id LIMIT 1) UNION ALL (SELECT * FROM test_for_append ORDER BY id desc LIMIT 1);

-- Blocks seq scan even if the first part of the append is an index scan.
CREATE INDEX test_for_append_idx ON test_for_append (id);
EXPLAIN (COSTS OFF) (SELECT * FROM test_for_append ORDER BY id LIMIT 1) UNION ALL (SELECT * FROM test_for_append ORDER BY random() LIMIT 1);
(SELECT * FROM test_for_append ORDER BY id LIMIT 1) UNION ALL (SELECT * FROM test_for_append ORDER BY random() LIMIT 1);

DROP TABLE test_for_append;
