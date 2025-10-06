-- Test partitioning support
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;

CREATE TABLE partitioned_foo (id bigint) PARTITION BY RANGE (id);
CREATE TABLE partitioned_foo_1 PARTITION OF partitioned_foo FOR VALUES FROM (1) TO (5);
CREATE TABLE partitioned_foo_2 PARTITION OF partitioned_foo FOR VALUES FROM (5) TO (11);

SET pg_no_seqscan.check_tables = 'partitioned_foo';

-- Querying parent should error
EXPLAIN (COSTS OFF) SELECT * FROM partitioned_foo;
SELECT * FROM partitioned_foo;

-- cleanup
Drop table partitioned_foo;