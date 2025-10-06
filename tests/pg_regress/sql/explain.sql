-- Test that EXPLAIN (COSTS OFF) queries are ignored
LOAD 'pg_no_seqscan';
SET pg_no_seqscan.level = ERROR;

-- To produce stable regression, inspired from:
-- https://github.com/postgres/postgres/blob/master/src/test/regress/sql/explain.sql

create function explain_filter(text) returns setof text
language plpgsql as
$$
declare
ln text;
begin
for ln in execute $1
    loop
        -- Replace any numeric word with just 'N'
        ln := regexp_replace(ln, '-?\m\d+\M\.?\d*', 'N', 'g');
        -- In sort output, the above won't match units-suffixed numbers
        ln := regexp_replace(ln, '\m\d+kB', 'NkB', 'g');
        -- Ignore text-mode buffers output because it varies depending
        -- on the system state
CONTINUE WHEN (ln ~ ' +Buffers: .*');
        -- Ignore text-mode "Planning:" line because whether it's output
        -- varies depending on the system state
CONTINUE WHEN (ln = 'Planning:');
        return next ln;
end loop;
end;
$$;

CREATE TABLE test_explain AS (SELECT * FROM generate_series(1,10) AS id);


-- EXPLAIN (COSTS OFF) should not trigger errors
EXPLAIN (COSTS OFF) SELECT * FROM test_explain;

-- EXPLAIN (COSTS OFF) ANALYZE should not trigger errors
select explain_filter('EXPLAIN (ANALYZE, COSTS OFF, TIMING OFF, SUMMARY OFF, BUFFERS OFF) SELECT * FROM test_explain;');

-- But regular query should trigger error
SELECT * FROM test_explain;

-- cleanup
drop table test_explain;
