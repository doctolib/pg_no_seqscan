# pg_no_seqscan

PG extension to prevent seqscan on dev environment

## Setup

- Follow the System Requirements in [pgrx instructions](https://github.com/pgcentralfoundation/pgrx).
- Install cargo-pgrx sub-command: `cargo install --locked cargo-pgrx`
- Initialize pgrx home directory: `cargo pgrx init --pg15 download`
- Run a PG with the extension `cargo pgrx run`

## Settings

- `pg_no_seqscan.ignored_schemas` to support a list of schemas to ignore when checking seqscan, useful to ignore internal schemas such as `pg_catalog` or `information_schema`
- `pg_no_seqscan.ignored_users` to support a list of users to ignore when checking seqscan, useful to ignore users that run migrations
- `pg_no_seqscan.level` to define behavior when a sequential scan occurs. Values can be: `off` (useful for pausing the extension), `warn` (log in postgres), `error` (postgres error)
- Sequential scans can be ignored on any query that contains the following comment: `pg_no_seqscan_skip`

## Motivation

The use of `Seqscan` (sequential scan) can have dramatic consequences and undesirable side effects on a production database. Sequential scans can lead to degraded performance, longer response times, and increased load on the database server. These issues can, in turn, affect the availability and responsiveness of applications that depend on the database.

We have observed multiple instances where unintended `Seqscan` occurred in production, despite our efforts to prevent them. These incidents have highlighted the limitations of our current approaches to avoiding sequential scans.

Training developers to avoid `Seqscan` in their SQL queries is an important step, but it is not sufficient to ensure that no `Seqscan` will occur in production. Human errors, changes in database schemas, and application evolutions can all contribute to the unintentional introduction of `Seqscan`.

Moreover, manually reviewing the query plans of every query run in production is not a realistic solution. This approach would be extremely time-consuming and difficult to maintain, especially in environments where thousands of queries are executed daily.

For these reasons, it is crucial to implement automated and robust mechanisms to detect and prevent `Seqscan` in production. This will ensure optimal performance and minimize the risk of incidents related to sequential scans.
