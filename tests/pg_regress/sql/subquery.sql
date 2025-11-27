-- Test subquery detection
select Substr(setting, 1, 2) = '18' from pg_settings where name = 'server_version_num'; -- output changed slightly from PG18
-- Setup
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;
SET enable_seqscan = off;
CREATE TABLE test_subq AS (SELECT * FROM generate_series(1,10) as id);

CREATE INDEX test_subq_idx ON test_subq(id) where id = 2;

-- Blocks query execution as a seqscan occur in first branch
EXPLAIN (COSTS OFF)
SELECT * FROM test_subq where id = 2
EXCEPT
SELECT * FROM test_subq;

SELECT * FROM test_subq where id = 2
EXCEPT
SELECT * FROM test_subq;

-- Blocks query execution as a seqscan occur in second branch
EXPLAIN (COSTS OFF)
SELECT * FROM test_subq
EXCEPT
SELECT * FROM test_subq where id = 2;

SELECT * FROM test_subq
EXCEPT
SELECT * FROM test_subq where id = 2;


-- Cleanup
DROP TABLE test_subq;
