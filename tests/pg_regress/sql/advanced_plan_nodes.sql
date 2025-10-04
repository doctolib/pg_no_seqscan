-- Test detection of sequential scans in Append
LOAD 'pg_no_seqscan';LOAD 'pg_no_seqscan';

-- Test MergeAppend (UNION ALL with ORDER BY on partitioned table)
CREATE TABLE test_merge_parent (id int, data text);
CREATE TABLE test_merge_child1 (CHECK (id < 2)) INHERITS (test_merge_parent);
CREATE TABLE test_merge_child2 (CHECK (id >= 2)) INHERITS (test_merge_parent);

INSERT INTO test_merge_parent SELECT i, 'data' || i FROM generate_series(1, 5) i;

-- This should trigger MergeAppend and detect sequential scans in child tables
EXPLAIN (COSTS OFF) SELECT * FROM test_merge_parent ORDER BY id LIMIT 5;

SELECT * FROM test_merge_parent ORDER BY id LIMIT 5;

DROP TABLE test_merge_child1, test_merge_child2, test_merge_parent;
