-- Test partitioning support
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
CREATE TABLE partitioned_foo (id bigint) PARTITION BY RANGE (id);
CREATE TABLE partitioned_foo_1 PARTITION OF partitioned_foo FOR VALUES FROM (1) TO (5);
CREATE TABLE partitioned_foo_2 PARTITION OF partitioned_foo FOR VALUES FROM (5) TO (11);

SET pg_no_seqscan.check_tables = 'partitioned_foo';

-- Blocks query execution and report seqscan on the parent table as a seqscan is made on each partition
EXPLAIN (COSTS OFF) SELECT * FROM partitioned_foo;
SELECT * FROM partitioned_foo;

-- Blocks query execution and report seqscan on the parent table even when querying directly a partition
EXPLAIN (COSTS OFF) SELECT * FROM partitioned_foo_1;
SELECT * FROM partitioned_foo_1;

RESET pg_no_seqscan.check_tables;
SET pg_no_seqscan.ignore_tables = 'partitioned_foo';

-- Allows query execution with seqscan on the parent table as the partitioned table is ignored
SELECT * FROM partitioned_foo;
-- Allows query execution with seqscan on the partition as the partitioned table is ignored
SELECT * FROM partitioned_foo_1;

-- cleanup
RESET pg_no_seqscan.ignore_tables;
DROP TABLE partitioned_foo cascade;
