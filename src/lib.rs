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
    use crate::hooks::HOOK_OPTION;
    use pgrx::prelude::*;

    #[pg_test]
    fn ignores_on_seqscan_when_level_off() {
        Spi::run("SET pg_no_seqscan.level = Off").expect("Unable to set settings");
        Spi::run("create table foo as (select * from generate_series(1,10));")
            .expect("Setup failed");

        Spi::run("select * from foo;").unwrap();

        unsafe {
            assert_eq!(
                HOOK_OPTION.as_mut().unwrap().tables_in_seqscans,
                Vec::<String>::new()
            );
        }
    }

    #[pg_test]
    #[should_panic(expected = "A 'Sequential Scan' on foo has been detected.")]
    fn panics_on_seqscan_when_level_error() {
        Spi::run("SET pg_no_seqscan.level = Error").expect("Unable to set settings");
        Spi::run("create table foo as (select * from generate_series(1,10));")
            .expect("Setup failed");

        Spi::run("select * from foo;").unwrap();
    }

    #[pg_test]
    fn warns_on_seqscan_when_level_warn() {
        Spi::run("SET pg_no_seqscan.level = Warn").expect("Unable to set settings");
        Spi::run("create table foo as (select * from generate_series(1,10));")
            .expect("Setup failed");

        Spi::run("select * from foo;").unwrap();

        unsafe {
            assert_eq!(
                HOOK_OPTION.as_mut().unwrap().tables_in_seqscans,
                vec!["foo".to_string()]
            );
        }
    }

    #[pg_test]
    fn detects_seqscan_on_multiple_selects() {
        Spi::run("SET pg_no_seqscan.level = WARN").expect("Unable to set settings");
        Spi::run(
            "create table foo as (select * from generate_series(1,10));
create table bar as (select * from generate_series(1,10));",
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
        unsafe {
            assert_eq!(
                HOOK_OPTION.as_mut().unwrap().tables_in_seqscans,
                vec!["bar".to_string()]
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
