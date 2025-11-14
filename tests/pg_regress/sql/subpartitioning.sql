-- Test partitioning support
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
SET enable_seqscan = off;
CREATE TABLE partitioned_foo (id bigint) PARTITION BY RANGE (id);
CREATE TABLE partitioned_foo_1 PARTITION OF partitioned_foo FOR VALUES FROM (1) TO (5);
CREATE TABLE partitioned_foo_2 PARTITION OF partitioned_foo FOR VALUES FROM (5) TO (11) PARTITION BY RANGE (id);
CREATE TABLE partitioned_foo_2_1 PARTITION OF partitioned_foo_2 FOR VALUES FROM (5) TO (8);
CREATE TABLE partitioned_foo_2_2 PARTITION OF partitioned_foo_2 FOR VALUES FROM (8) TO (11);

INSERT INTO partitioned_foo SELECT i FROM generate_series(1, 10) i;

CREATE INDEX on partitioned_foo_1 USING btree (id);

-- show data distribution
SELECT id, tableoid::regclass from partitioned_foo ORDER BY id /*pg_no_seqscan_skip*/;

-- Blocks query execution as no table settings are defined
EXPLAIN (COSTS OFF) SELECT id, tableoid::regclass from partitioned_foo ORDER BY id;
SELECT id, tableoid::regclass from partitioned_foo ORDER BY id;

-- Blocks query execution as root table appears in check_tables settings
SET pg_no_seqscan.check_tables = 'partitioned_foo';
SELECT id, tableoid::regclass from partitioned_foo ORDER BY id;

-- Allows query execution as root table appears in ignore_tables settings
RESET pg_no_seqscan.check_tables;
SET pg_no_seqscan.ignore_tables = 'partitioned_foo';
SELECT id, tableoid::regclass from partitioned_foo ORDER BY id;

-- Cleanup
DROP TABLE partitioned_foo cascade;
