mod guc;
mod helpers;
mod hooks;

use pgrx::prelude::*;

::pgrx::pg_module_magic!();

#[pg_guard]
pub extern "C" fn _PG_init() {
    unsafe { hooks::init_hooks() };
    guc::register_gucs();
}

#[cfg(any(test, feature = "pg_test"))]
#[allow(static_mut_refs)]
#[pg_schema]
mod tests {
    use crate::{guc::DetectionLevelEnum, hooks::HOOK_OPTION};
    use pgrx::prelude::*;

    #[pg_test]
    fn ignores_on_seqscan_when_level_off() {
        set_pg_seq_scan_level(DetectionLevelEnum::Off);

        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");

        Spi::run("select * from foo;").unwrap();

        assert_no_seq_scan();
    }

    #[pg_test]
    #[should_panic(expected = "A 'Sequential Scan' on foo has been detected.")]
    fn panics_on_seqscan_when_level_error() {
        set_pg_seq_scan_level(DetectionLevelEnum::Error);

        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");

        Spi::run("select * from foo;").unwrap();
    }

    #[pg_test]
    fn warns_on_seqscan_when_level_warn() {
        set_pg_seq_scan_level(DetectionLevelEnum::Warn);

        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");

        Spi::run("select * from foo;").unwrap();

        assert_seq_scan(vec!["foo".to_string()]);
    }

    #[pg_test]
    fn detects_seqscan_on_multiple_selects() {
        set_pg_seq_scan_level(DetectionLevelEnum::Warn);
        Spi::run(
            "create table foo as (select * from generate_series(1,10) as id);
create table bar as (select * from generate_series(1,10) as id);",
        )
        .expect("Setup failed");

        Spi::run("select * from foo;").unwrap();
        unsafe {
            assert_eq!(
                HOOK_OPTION.as_mut().unwrap().tables_in_seqscans,
                vec!["foo".to_string()]
            );
        }

        Spi::run("select * from bar;").unwrap();

        assert_seq_scan(vec!["bar".to_string()]);
    }

    #[pg_test]
    fn ignores_on_seqscan_when_explain() {
        set_pg_seq_scan_level(DetectionLevelEnum::Warn);
        Spi::run("create table foo as (select * from generate_series(1,10) as id);")
            .expect("Setup failed");

        Spi::run("explain select * from foo;").unwrap();

        assert_no_seq_scan();
    }

    #[pg_test]
    fn does_nothing_when_query_by_pk() {
        set_pg_seq_scan_level(DetectionLevelEnum::Warn);
        Spi::run(
            "create table foo (id bigint PRIMARY KEY);
             insert into foo SELECT generate_series(1,10);
        ",
        )
        .expect("Setup failed");

        Spi::run("select * from foo where id=1;").unwrap();

        assert_no_seq_scan();
    }

    fn set_pg_seq_scan_level(detection_level: DetectionLevelEnum) {
        let level = match detection_level {
            DetectionLevelEnum::Warn => "WARN",
            DetectionLevelEnum::Error => "ERROR",
            DetectionLevelEnum::Off => "OFF",
        };

        let set_level = format!("SET pg_no_seqscan.level = {}", level);
        Spi::run(&set_level).expect("Unable to set settings");
    }

    fn assert_seq_scan(table_vec: Vec<String>) {
        unsafe {
            assert_eq!(HOOK_OPTION.as_mut().unwrap().tables_in_seqscans, table_vec);
        }
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
