mod guc;
mod helpers;
mod hooks;

use pgrx::prelude::*;

::pgrx::pg_module_magic!();

#[pg_guard]
pub extern "C-unwind" fn _PG_init() {
    guc::register_gucs();
    unsafe { hooks::init_hooks() };
}

#[cfg(any(test, feature = "pg_test"))]
#[allow(static_mut_refs)]
#[pg_schema]
mod tests {
    use crate::{guc::DetectionLevelEnum, hooks::HOOK_OPTION};
    use pgrx::prelude::*;
    use std::panic;

    fn set_pg_no_seqscan_level(detection_level: DetectionLevelEnum) {
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

    fn assert_seq_scan_error(query: &str, table_vec: Vec<String>) {
        assert!(panic::catch_unwind(|| Spi::run(query)).is_err());
        assert_seq_scan(table_vec);
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
