mod guc;
mod helpers;
mod hooks;

use pgrx::prelude::*;

::pgrx::pg_module_magic!();

#[pg_guard]
pub extern "C-unwind" fn _PG_init() {
    unsafe { hooks::init_hooks() };
    guc::register_gucs();
}

#[cfg(any(test, feature = "pg_test"))]
#[allow(static_mut_refs)]
#[pg_schema]
mod tests {
    use crate::{guc::DetectionLevelEnum, hooks::HOOK_OPTION};
    use pgrx::prelude::*;
    use std::panic;

    #[pg_test]
    fn check_only_schemas_in_settings() {
        set_pg_no_seqscan_level(DetectionLevelEnum::Error);

        Spi::run(
            "
            CREATE SCHEMA test_schema1;
            CREATE SCHEMA test_schema2;
            CREATE TABLE test_schema1.foo AS (SELECT * FROM generate_series(1,10) as id);
            CREATE TABLE test_schema2.bar AS (SELECT * FROM generate_series(1,10) as id);
            CREATE TABLE public.baz AS (SELECT * FROM generate_series(1,10) as id);
        ",
        )
        .expect("Setup failed");

        set_check_schemas(vec!["test_schema1", "public"]);

        // These should be ignored due to schema not in check_schemas setting
        Spi::run("SELECT * FROM test_schema2.bar;").unwrap();

        // This should error due to seqscan in checked schema
        assert_seq_scan_error("SELECT * FROM test_schema1.foo;", vec!["foo".to_string()]);
        assert_seq_scan_error("SELECT * FROM public.baz;", vec!["baz".to_string()]);
        assert_seq_scan_error("SELECT * FROM baz;", vec!["baz".to_string()]);
    }

    #[pg_test]
    fn ignores_on_seqscan_when_level_off() {
        set_pg_no_seqscan_level(DetectionLevelEnum::Off);

        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");

        Spi::run("select * from foo;").unwrap();

        assert_no_seq_scan();
    }

    #[pg_test]
    fn panics_on_seqscan_when_level_error() {
        set_pg_no_seqscan_level(DetectionLevelEnum::Error);

        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");
        assert_seq_scan_error("select * from foo;", vec!["foo".to_string()]);
    }

    #[pg_test]
    fn warns_on_seqscan_when_level_warn() {
        set_pg_no_seqscan_level(DetectionLevelEnum::Warn);

        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");

        Spi::run("select * from foo;").unwrap();
        assert_seq_scan(vec!["foo".to_string()]);
    }

    #[pg_test]
    fn detects_seqscan_on_multiple_selects() {
        set_pg_no_seqscan_level(DetectionLevelEnum::Error);
        Spi::run(
            "create table foo as (select * from generate_series(1,10) as id);
    create table bar as (select * from generate_series(1,10) as id);",
        )
        .expect("Setup failed");

        assert_seq_scan_error("select * from foo;", vec!["foo".to_string()]);
        assert_seq_scan_error("select * from bar;", vec!["bar".to_string()]);
    }

    #[pg_test]
    fn ignores_seqscan_on_query_with_ignore_comment() {
        set_pg_no_seqscan_level(DetectionLevelEnum::Error);
        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");

        Spi::run("select * from foo /* pg_no_seqscan_skip */;").unwrap();
        Spi::run("select * from foo /* host_name:a-b-1.2.foo,db:my_database,git:0123456789abcdef,pg_no_seqscan_skip,path:/foo/source.java:108`(<>)' */;").unwrap();
        Spi::run("select * from foo /*pg_no_seqscan_skip*/;").unwrap();
    }

    #[pg_test]
    fn ignores_on_seqscan_when_explain() {
        set_pg_no_seqscan_level(DetectionLevelEnum::Error);
        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");

        Spi::run("explain select * from foo;").unwrap();
    }

    #[pg_test]
    fn ignores_on_seqscan_when_explain_analyze() {
        set_pg_no_seqscan_level(DetectionLevelEnum::Error);
        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");

        Spi::run("explain analyze select * from foo;").unwrap();
        assert_no_seq_scan();
    }

    #[pg_test]
    fn ignores_on_ignore_users() {
        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");

        Spi::run("CREATE USER test_user").expect("failed to create user");
        set_ignore_users(vec!["test_user_2", "test_user"]);

        Spi::run("GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE foo TO test_user")
            .expect("failed to grant access to test_user");

        Spi::run("SET SESSION AUTHORIZATION test_user")
            .expect("failed to set session authorization");

        Spi::run(" select * from foo;").unwrap();

        assert_no_seq_scan();
    }

    #[pg_test]
    fn ignores_on_ignore_tables() {
        Spi::run(
            "create table foo as (select * from generate_series(1,10) as id);
            create table bar as (select * from generate_series(1,10) as id);
            create table baz as (select * from generate_series(1,10) as id);
            ",
        )
        .expect("Setup failed");

        Spi::run("SET pg_no_seqscan.ignore_tables = 'something,foo,baz'")
            .expect("Unable to set ignore_tables");

        Spi::run("select * from foo;").unwrap();
        Spi::run("select * from baz;").unwrap();
        assert_seq_scan_error("select * from bar;", vec!["bar".to_string()]);
    }

    #[pg_test]
    fn checks_on_check_tables() {
        Spi::run(
            "create table foo as (select * from generate_series(1,10) as id);
            create table bar as (select * from generate_series(1,10) as id);
            create table baz as (select * from generate_series(1,10) as id);
            ",
        )
        .expect("Setup failed");

        Spi::run("SET pg_no_seqscan.check_tables = 'something,foo,baz'")
            .expect("Unable to set check_tables");

        Spi::run("select * from bar;").unwrap();
        assert_seq_scan_error("select * from foo;", vec!["foo".to_string()]);
        assert_seq_scan_error("select * from baz;", vec!["baz".to_string()]);
    }

    #[pg_test]
    fn ignores_ignore_tables_option_if_check_tables_option_is_set() {
        Spi::run(
            "create table foo as (select * from generate_series(1,10) as id);
            create table bar as (select * from generate_series(1,10) as id);
            create table baz as (select * from generate_series(1,10) as id);
            ",
        )
        .expect("Setup failed");

        Spi::run("SET pg_no_seqscan.check_tables = 'something,foo,baz'")
            .expect("Unable to set check_tables");

        Spi::run("SET pg_no_seqscan.ignore_tables = 'something,foo,baz'")
            .expect("Unable to set ignore_tables");

        Spi::run("select * from bar;").unwrap();
        assert_seq_scan_error("select * from foo;", vec!["foo".to_string()]);
        assert_seq_scan_error("select * from baz;", vec!["baz".to_string()]);
    }

    #[pg_test]
    fn check_all_databases_when_check_database_is_not_defined() {
        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");

        Spi::run("SET pg_no_seqscan.check_databases = '';").expect("Unable to set check_databases");
        assert_seq_scan_error("select * from foo;", vec!["foo".to_string()]);
    }

    #[pg_test]
    fn ignores_seqscan_on_db_not_defined_in_check_databases() {
        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");

        Spi::run("SET pg_no_seqscan.check_databases = 'postgres';")
            .expect("Unable to set check_databases");
        Spi::run("select * from foo;").unwrap();
        assert_no_seq_scan();
    }

    #[pg_test]
    fn detects_seqscan_on_db_defined_in_check_databases() {
        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");

        Spi::run("SET pg_no_seqscan.check_databases = 'pgrx_tests';")
            .expect("Unable to set check_databases");
        assert_seq_scan_error("select * from foo;", vec!["foo".to_string()]);

        Spi::run("SET pg_no_seqscan.check_databases = 'postgres,pgrx_tests';")
            .expect("Unable to set check_databases");
        assert_seq_scan_error("select * from foo;", vec!["foo".to_string()]);

        Spi::run("SET pg_no_seqscan.check_databases = 'pgrx_tests,postgres';")
            .expect("Unable to set check_databases");
        assert_seq_scan_error("select * from foo;", vec!["foo".to_string()]);
    }

    #[pg_test]
    fn detects_seqscan_after_explain_analyze() {
        set_pg_no_seqscan_level(DetectionLevelEnum::Error);
        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");

        Spi::run("explain analyze select * from foo;").unwrap();
        assert_no_seq_scan();
        assert_seq_scan_error("select * from foo;", vec!["foo".to_string()]);
    }

    #[pg_test]
    fn does_nothing_when_query_by_pk() {
        set_pg_no_seqscan_level(DetectionLevelEnum::Error);
        Spi::run(
            "create table foo (id bigint PRIMARY KEY);
             insert into foo SELECT generate_series(1,10);
        ",
        )
        .expect("Setup failed");

        Spi::run("select * from foo where id=1;").unwrap();
        assert_no_seq_scan();
    }

    #[pg_test]
    fn does_nothing_when_querying_a_sequence() {
        set_pg_no_seqscan_level(DetectionLevelEnum::Error);
        Spi::run("CREATE SEQUENCE foo_seq;").expect("Setup failed");

        Spi::run("select last_value from foo_seq;").unwrap();
        assert_no_seq_scan();
    }

    fn set_pg_no_seqscan_level(detection_level: DetectionLevelEnum) {
        let level = match detection_level {
            DetectionLevelEnum::Warn => "WARN",
            DetectionLevelEnum::Error => "ERROR",
            DetectionLevelEnum::Off => "OFF",
        };

        let set_level = format!("SET pg_no_seqscan.level = {}", level);
        Spi::run(&set_level).expect("Unable to set settings");
    }

    fn set_ignore_users(users: Vec<&str>) {
        let users_list = users.join(",");
        let set_ignore_users = format!("SET pg_no_seqscan.ignore_users = '{}'", users_list);
        Spi::run(&set_ignore_users).expect("Unable to set ignore_users");
    }

    fn set_check_schemas(schemas: Vec<&str>) {
        let schemas_list = schemas.join(",");
        let set_check_schemas = format!("SET pg_no_seqscan.check_schemas = '{}'", schemas_list);
        Spi::run(&set_check_schemas).expect("Unable to set check_schemas");
    }

    fn assert_seq_scan(table_vec: Vec<String>) {
        unsafe {
            assert_eq!(HOOK_OPTION.as_mut().unwrap().tables_in_seqscans, table_vec);
        }
    }

    fn assert_seq_scan_error(query: &str, table_vec: Vec<String>) {
        assert!(panic::catch_unwind(|| Spi::run(query)).is_err());
        assert_seq_scan(table_vec);
    }

    fn assert_no_seq_scan() {
        unsafe {
            assert_eq!(
                HOOK_OPTION.as_mut().unwrap().tables_in_seqscans,
                Vec::<String>::new()
            );
        }
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
