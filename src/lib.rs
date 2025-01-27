mod hooks;

use pgrx::prelude::*;

::pgrx::pg_module_magic!();

#[pg_guard]
pub extern "C" fn _PG_init() {
    unsafe { hooks::init_hooks() };
}

#[cfg(any(test, feature = "pg_test"))]
#[allow(static_mut_refs)]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;
    use std::collections::HashSet;
    use crate::hooks::HOOK_OPTION;


    #[pg_test]
    fn detects_seqscan_on_select() {
        Spi::run("
        create table foo as (select * from generate_series(1,10));
        select * from foo;
        ").unwrap();
        unsafe {
            assert_eq!(HOOK_OPTION.as_mut().unwrap().tables_in_seqscans, HashSet::from_iter(vec!["foo".to_string()]));
        }   
    }

    #[pg_test]
    fn detects_seqscan_on_multiple_selects() {
        Spi::run("
        create table foo as (select * from generate_series(1,10));
        select * from foo;
        ").unwrap();
        unsafe {
            assert_eq!(HOOK_OPTION.as_mut().unwrap().tables_in_seqscans, HashSet::from_iter(vec!["foo".to_string()]));
        }   
        Spi::run("
        create table bar as (select * from generate_series(1,10));
        select * from bar;
        ").unwrap();
        unsafe {
            assert_eq!(HOOK_OPTION.as_mut().unwrap().tables_in_seqscans, HashSet::from_iter(vec!["bar".to_string()]));
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
