-- Test schema filtering

SET pg_no_seqscan.level = ERROR;

CREATE SCHEMA test_schema1;
CREATE SCHEMA test_schema2;
CREATE TABLE test_schema1.foo AS (SELECT * FROM generate_series(1,10) as id);
CREATE TABLE test_schema2.bar AS (SELECT * FROM generate_series(1,10) as id);
CREATE TABLE public.baz AS (SELECT * FROM generate_series(1,10) as id);

-- Set check_schemas to only check test_schema1 and public
SET pg_no_seqscan.check_schemas = 'test_schema1,public';

-- This should be ignored due to schema not in check_schemas setting
SELECT * FROM test_schema2.bar;

-- These should error
SELECT * FROM test_schema1.foo;

-- Cleanup
DROP TABLE test_schema1.foo, test_schema2.bar, public.baz;
DROP SCHEMA test_schema1, test_schema2;
