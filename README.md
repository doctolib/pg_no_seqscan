# pg_no_seqscan

PG extension to prevent seqscan on dev environment. This can help in dev environment and CI to identify missing indexes
that could cause dramatic performance in production.

> ⚠️ Some important notes:
> - This extension is not meant to be used in production.
> - This extension is not magic and will not detect all performance issues.
> - Sometimes a sequential scan is more efficient than an index scan.
> - Having an index scan is not a guarantee of performance [(nice article on the topic)](https://www.pgmustard.com/blog/index-scan-doesnt-mean-its-fast)

## How it works

- Using `enable_seqscan = off` to discourage PostgreSQL from using sequential scans and prefer an index, even if the
  table is empty
- The extension parses the query plan and checks if there are any nodes with a sequential scan
- If any are found, a notice message is shown or an exception is raised and the query fails (depending on the extension
  settings)

## How to use it

### Pre-requisites

- Follow the System Requirements in [pgrx instructions](https://github.com/pgcentralfoundation/pgrx).
- Install cargo-pgrx sub-command: `cargo install --locked cargo-pgrx`
- Initialize pgrx home directory: `cargo pgrx init --pg15 download`

### Build the extension

For now, you need to build the extension locally:
`cargo build -r`
That will generate the following files:

- pg_no_seqscan.control
- pg_no_seqscan.so
- pg_no_seqscan--$VERSION.sql

### Load and configure the extension in a local PG database

1. check the share and pkglib directories of your favorite postgres database with:

```bash
export PG_SHAREDIR=$(pg_config --sharedir)
export PG_PKGLIBDIR=$(pg_config --pkglibdir)
```

2. copy the generated files:
    - pg_no_seqscan.so goes to `$PG_PKGLIBDIR` directory
    - pg_no_seqscan.control goes to `$PG_SHAREDIR/extension` directory
3. change the postgresql.conf (`show config_file` will tell you where it's located), and add:

```
# Required settings for pg_no_seqscan:
shared_preload_libraries = 'pg_no_seqscan.so' # load pg_no_seqscan extension
enable_seqscan = 'off'                        # discourage seqscans
jit_above_cost = 40000000000                  # avoids to use jit on each query, as the cost becomes much higher with
                                              # enable_seqscan off

# Optional settings for pg_no_seqscan:
#pg_no_seqscan.check_databases = ''           # Databases to check seqscan for, comma separated.
                                              # If empty, all databases will be checked.

#pg_no_seqscan.check_schemas = 'public'       # Schemas to check seqscan for, comma separated.
                                              # If empty, all schemas will be checked.

#pg_no_seqscan.check_tables = ''              # Tables to check seqscan for, comma separated.
                                              # Useful when only wanting to check some tables.
                                              # If empty, all tables will be checked.

#pg_no_seqscan.ignore_users = ''              # Users to ignore, comma separated.
                                              # Useful to ignore:
                                              #   - users that run migrations (legitimate seqscan)
                                              #   - pg clients like IDEs, that are doing seqscan when displaying
                                              #     the content of a table

#pg_no_seqscan.ignore_tables = ''             # Tables to ignore, comma separated.
                                              # Useful for tables that will remain small and that do not need any index.
                                              # This setting is ignored if some tables are declared in `check_tables`.

#pg_no_seqscan.level = 'Error'                # Detection level for sequential scans:
                                              #   Error: force query to fail when the query will cause a seqscan
                                              #   Warn: a notice is displayed on seqscan, available in pg logs
                                              #   Off: detection skipped, could be use to pause the extension
```

If you need, uncomment these settings to use the value of your preference.

4. restart the server
5. run: `CREATE EXTENSION pg_no_seqscan;`

### pg_no_seqscan is now ready

```postgresql
CREATE TABLE foo AS (SELECT generate_series(1, 10000) AS id);

SELECT * FROM foo WHERE id = 123;
ERROR:  A 'Sequential Scan' on foo has been detected.
  - Run an
EXPLAIN on your query to check the query plan. - Make sure the query is compatible
with the existing indexes.

Query: SELECT * FROM foo WHERE id = 123;

CREATE INDEX foo_id_idx ON foo (id);

SELECT * FROM foo WHERE id = 123;
id  
-----
 123

SELECT * FROM foo LIMIT 3;
ERROR:  A 'Sequential Scan' on foo has been detected.
  - Run an
EXPLAIN on your query to check the query plan. - Make sure the query is compatible
with the existing indexes.

SELECT * FROM foo LIMIT 3 /* pg_no_seqscan_skip */;
id 
----
  1
  2
  3
```

Notes:

- as mentioned in the example, sequential scans will be ignored on any query that contains the following comment:
  `pg_no_seqscan_skip`
- it's possible to override the settings in the current session by using `SET <setting_name> = <setting value>`, and to
  show them with `SHOW <setting_name>`. As a reminder settings are:
    - `enable_seqscan`
    - `jit_above_cost` 
    - `pg_no_seqscan.check_databases`
    - `pg_no_seqscan.check_schemas`
    - `pg_no_seqscan.ignore_users`
    - `pg_no_seqscan.ignore_tables`
    - `pg_no_seqscan.level`

## Motivation

### Why seqscans can be problematic

When retrieving data from a table, one of the two most frequent strategies are:

- sequential scans (seqscans), reading directly each tuple of the table until some conditions are met
- index scans, browsing an index to find the rows that are relevant and then retrieving the related data in the table.

In some cases sequential scans could be faster than index scans:

- table is very small
- the query needs to read a high percentage of the tuples of the table
  In such situations, browsing an index will require to read both most of the index pages and most of the table pages,
  where an seq scan would directly fetch the appropriate tuples and be more efficient.

In most of the other situations, sequential scan could be a symptom of a missing index. To filter the table, postgres
has no other choice than filtering directly the rows in the table, and when the table is large, that could cause:

- intensive I/Os
- intensive CPU
- slow query response time for the current query
- slow query response time for other queries, as the database is consuming too many resources

These issues can, in turn, affect the availability and responsiveness of production applications that rely on the
database.

### Looking for a strategy to prevent slow seqscan in production

We have observed multiple instances where unintended `Seqscan` occurred in production, despite our efforts to prevent
them. These incidents have highlighted the limitations of our current approaches to avoiding sequential scans.

Training developers to avoid `Seqscan` in their SQL queries is an important step, but it is not sufficient to ensure
that no `Seqscan` will occur in production. Here are several factors that could lead to seqscans:

- Dataset: seqscans are expected locally as the data set is small. Having a local dataset that represents the production
  could be challenging (due to data volume and anonymization requirements) 
- Lack of training
- Application evolution (changing a filter or an ORDER BY clause for example)
- Human errors

Moreover, manually reviewing the query plans of every query running in production is not a realistic solution. This
approach would be extremely time-consuming and difficult to maintain, especially in environments where thousands of
different queries are executed daily.

For these reasons, it is crucial to implement automated and robust mechanisms to detect and prevent `Seqscan` in
production. This will ensure optimal performance and minimize the risk of incidents related to sequential scans.

### Benchmark

According to a basic benchmark, the overhead of pg_no_seqscan should not bother your CI response time:
`docker exec -it benchmark pgbench -T300 -r postgres://postgres@localhost/postgres`

|                                                                                                                        | Without the extension | With the extension |
|------------------------------------------------------------------------------------------------------------------------|-----------------------|--------------------|
| scaling factor                                                                                                         | 1                     | 1                  |
| query mode                                                                                                             | simple                | simple             |
| number of clients                                                                                                      | 1                     | 1                  |
| number of threads                                                                                                      | 1                     | 1                  |
| maximum number of tries                                                                                                | 1                     | 1                  |
| duration                                                                                                               | 300 s                 | 300 s              |
| number of transactions actually processed                                                                              | 357190                | 344692             |
| number of failed transactions                                                                                          | 0 (0.000%)            | 0 (0.000%)         |
| latency average                                                                                                        | 0.840 ms              | 0.870 ms           |
| initial connection time                                                                                                | 5.165 ms              | 5.159 ms           |
| tps (without initial connection time)                                                                                  | 1190.651554           | 1148.989580        |
| statement latencies in milliseconds and failures:                                                                      |                       |                    |
| `\set aid random(1, 100000 * :scale)`                                                                                  | 0.001                 | 0.001              |
| `\set bid random(1, 1 * :scale) `                                                                                      | 0.000                 | 0.000              |
| `\set tid random(1, 10 * :scale)`                                                                                      | 0.000                 | 0.000              |
| `\set delta random(-5000, 5000) `                                                                                      | 0.000                 | 0.000              |
| `BEGIN;                         `                                                                                      | 0.040                 | 0.042              |
| `UPDATE pgbench_accounts <br/>SET abalance = abalance + :delta <br/>WHERE aid = :aid;`                                 | 0.112                 | 0.117              |
| `SELECT abalance <br/>FROM pgbench_accounts <br/>WHERE aid = :aid;                   `                                 | 0.081                 | 0.086              |
| `UPDATE pgbench_tellers <br/>SET tbalance = tbalance + :delta <br/>WHERE tid = :tid; `                                 | 0.086                 | 0.089              |
| `UPDATE pgbench_branches <br/>SET bbalance = bbalance + :delta <br/>WHERE bid = :bid;`                                 | 0.081                 | 0.088              |
| `INSERT INTO pgbench_history (tid, bid, aid, delta, mtime) <br/>VALUES (:tid, :bid, :aid, :delta, CURRENT_TIMESTAMP);` | 0.072                 | 0.076              |
| `END;                                                                                                           `      | 0.364                 | 0.369              |

Note that performance could differ:
- when the query plans becomes more complex (but often the query execution is much longer)
- when some of the pg_no_seqscan settings contain a long list of values
