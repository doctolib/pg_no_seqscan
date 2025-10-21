-- Test partitioning support
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
SET enable_seqscan = off;

CREATE TABLE partitioned_foo (id bigint) PARTITION BY RANGE (id);
CREATE TABLE partitioned_foo_1 PARTITION OF partitioned_foo FOR VALUES FROM (1) TO (5);
CREATE TABLE partitioned_foo_2 PARTITION OF partitioned_foo FOR VALUES FROM (5) TO (11) PARTITION BY RANGE (id);
CREATE TABLE partitioned_foo_2_1 PARTITION OF partitioned_foo_2 FOR VALUES FROM (5) TO (8);
CREATE TABLE partitioned_foo_2_2 PARTITION OF partitioned_foo_2 FOR VALUES FROM (8) TO (11);

INSERT INTO partitioned_foo SELECT i FROM generate_series(1, 10) i;

select id, tableoid::regclass from partitioned_foo order by id /*pg_no_seqscan_skip*/;

create index on partitioned_foo_1 USING btree (id);

explain (costs off) select id, tableoid::regclass from partitioned_foo order by id;

-- when no rules are defined, the seq scan should be blocked
select id, tableoid::regclass from partitioned_foo order by id;

-- when only the root table is checked, seq scan should be blocked
SET pg_no_seqscan.check_tables = 'partitioned_foo';
select id, tableoid::regclass from partitioned_foo order by id;

reset pg_no_seqscan.level;
reset enable_seqscan;
DROP TABLE partitioned_foo cascade;