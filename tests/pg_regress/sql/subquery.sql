-- Test subquery detection
SET enable_seqscan = off;
CREATE TABLE test_subq AS (SELECT * FROM generate_series(1,10) as id);

CREATE INDEX test_subq_idx ON test_subq(id) where id = 2;

-- Other subquery with seaq scan on the right branch should fail
EXPLAIN (COSTS OFF)
SELECT * FROM test_subq where id = 2
EXCEPT
SELECT * FROM test_subq;

SELECT * FROM test_subq where id = 2
EXCEPT
SELECT * FROM test_subq;

-- Other subquery with seq scan on the left branch should fail
EXPLAIN (COSTS OFF)
SELECT * FROM test_subq
EXCEPT
SELECT * FROM test_subq where id = 2;

SELECT * FROM test_subq
EXCEPT
SELECT * FROM test_subq where id = 2;


-- Cleanup
DROP TABLE test_subq;
