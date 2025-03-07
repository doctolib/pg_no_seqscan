# pg_no_seqscan

PG extension to prevent seqscan on dev environment. This can help in dev environment and CI to identify missing indexes that could cause dramatic performance in production.

⚠️ This extension is not meant to be used in production.

## How it works

- Using `enable_seqscan = off` to discourage PostgreSQL from using sequential scans and prefer an index, even if the table is empty
- The extension parses the query plan and checks if there are any nodes with a sequential scan
- If any are found, a notice message is shown or an exception is raised and the query fails (depending on the extension settings)

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
shared_preload_libraries = 'pg_no_seqscan.so'                      # load pg_no_seqscan extension
enable_seqscan = 'off'                                             # discourage seqscans
jit_above_cost = 40000000000                                       # avoids to use jit on each query, as the cost becomes much higher with enable_seqscan off
# pg_no_seqscan.ignored_schemas = 'pg_catalog,information_schema'  # tables in this schema will be ignored
# pg_no_seqscan.ignored_users = ''                                 # users that will be ignored
# pg_no_seqscan.level = 'Error'                                    # Detection level for sequential scans
```
If you need, uncomment these settings to use the value of your preference:
- `pg_no_seqscan.ignored_schemas` to support a list of schemas to ignore when checking seqscan, useful to ignore internal schemas such as `pg_catalog` or `information_schema`
- `pg_no_seqscan.ignored_users` to support a list of users to ignore when checking seqscan, useful to ignore users that run migrations
- `pg_no_seqscan.ignored_tables` to support a list of tables to ignore when checking seqscan, useful for tables that will always be small
- `pg_no_seqscan.level` to define behavior when a sequential scan occurs. Values can be: `off` (useful for pausing the extension), `warn` (log in postgres), `error` (postgres error)
4. restart the server
5. run: `CREATE EXTENSION pg_no_seqscan;`

### pg_no_seqscan is now ready

```postgresql
create table foo as (select generate_series(1,10000) as id);

select * from foo where id = 123;
ERROR:  A 'Sequential Scan' on foo has been detected.
  - Run an EXPLAIN on your query to check the query plan.
    - Make sure the query is compatible with the existing indexes.

Query: select * from foo where id = 123;

CREATE INDEX foo_id_idx ON foo (id);

select * from foo where id = 123;
id  
-----
 123

select * from foo;
ERROR:  A 'Sequential Scan' on foo has been detected.
  - Run an EXPLAIN on your query to check the query plan.
    - Make sure the query is compatible with the existing indexes.

select * from foo LIMIT 10 /* pg_no_seqscan_skip */;
id 
----
  1
  2
  3
  4
  5
  6
  7
  8
  9
 10
```

Notes:
- as mentioned in the example, sequential scans will be ignored on any query that contains the following comment: `pg_no_seqscan_skip`
- it's possible to override the settings in the current session by using `SET <setting_name> = <setting value>`, and to show them with `SHOW <setting_name>`. As a reminder settings are:
  - `enable_seqscan`
  - `jit_above_cost` 
  - `pg_no_seqscan.ignored_schemas`
  - `pg_no_seqscan.ignored_users`
  - `pg_no_seqscan.level`

## Motivation

### Why seqscans can be problematic
When retrieving data from a table, one of the two most frequent strategies are:
- sequential scans (seqscans), reading directly each tuple of the table until some conditions are met
- index scans, browsing an index to find the rows that are relevant and then retrieving the related data in the table.

In some cases sequential scans could be faster than index scans:
- table is very small
- the query needs to read a high percentage of the tuples of the table
In such situations, browsing an index will require to read both most of the index pages and most of the table pages, where an seq scan would directly fetch the appropriate tuples and be more efficient.

In most of the other situations, sequential scan could be a symptom of a missing index. To filter the table, postgres has no other choice than filtering directly the rows in the table, and when the table is large, that could cause:
- intensive I/Os
- intensive CPU
- slow query response time for the current query
- slow query response time for other queries, as the database is consuming too many resources

These issues can, in turn, affect the availability and responsiveness of production applications that rely on the database.

### Looking for a strategy to prevent slow seqscan in production
We have observed multiple instances where unintended `Seqscan` occurred in production, despite our efforts to prevent them. These incidents have highlighted the limitations of our current approaches to avoiding sequential scans.

Training developers to avoid `Seqscan` in their SQL queries is an important step, but it is not sufficient to ensure that no `Seqscan` will occur in production. Here are several factors that could lead to seqscans:
- Dataset: seqscans are expected locally as the data set is small. Having a local dataset that represents the production could be challenging (due to data volume and anonymization requirements) 
- Lack of training
- Application evolution (changing a filter or an ORDER BY clause for example)
- Human errors

Moreover, manually reviewing the query plans of every query running in production is not a realistic solution. This approach would be extremely time-consuming and difficult to maintain, especially in environments where thousands of different queries are executed daily.

For these reasons, it is crucial to implement automated and robust mechanisms to detect and prevent `Seqscan` in production. This will ensure optimal performance and minimize the risk of incidents related to sequential scans.
