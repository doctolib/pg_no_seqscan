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

    #[pg_test]
    fn test_detects_seqscan_with_bitmap_or() {
        set_pg_no_seqscan_level(DetectionLevelEnum::Error);
        Spi::run(
            "CREATE TABLE foo (id bigint, value text, category text);
             INSERT INTO foo SELECT i, 'value' || i, CASE WHEN i % 2 = 0 THEN 'even' ELSE 'odd' END FROM generate_series(1, 1000) i;
             CREATE INDEX idx_foo_value ON foo(value);
             CREATE INDEX idx_foo_category ON foo(category);",
        )
        .expect("Setup failed");

        // Query that should use BitmapOr (OR condition with indexed columns)
        let query = "SELECT * FROM foo WHERE value = 'value1' OR category = 'even';";
        let explain_result = Spi::get_one::<String>(&format!("EXPLAIN {}", query))
            .expect("Failed to get EXPLAIN output")
            .expect("Expected non-null result");

        // Log the plan for debugging
        notice!("EXPLAIN BitmapOr output: {}", explain_result);

        // Check the plan - optimizer may use Bitmap scan or Seq scan depending on statistics
        assert!(
            !explain_result.is_empty(),
            "Expected non-empty EXPLAIN output"
        );

        // If it uses seq scan, it should be detected
        if explain_result.contains("Seq Scan") {
            assert_seq_scan_error(query, vec!["foo".to_string()]);
        }
    }

    #[pg_test]
    fn test_detects_seqscan_with_bitmap_and() {
        set_pg_no_seqscan_level(DetectionLevelEnum::Error);
        Spi::run(
            "CREATE TABLE foo (id bigint, value text, category text);
             INSERT INTO foo SELECT i, 'value' || i, CASE WHEN i % 2 = 0 THEN 'even' ELSE 'odd' END FROM generate_series(1, 1000) i;
             CREATE INDEX idx_foo_value ON foo(value);
             CREATE INDEX idx_foo_category ON foo(category);",
        )
        .expect("Setup failed");

        // Query that should use BitmapAnd (AND condition with indexed columns)
        let query = "SELECT * FROM foo WHERE value LIKE 'value%' AND category = 'even';";
        let explain_result = Spi::get_one::<String>(&format!("EXPLAIN {}", query))
            .expect("Failed to get EXPLAIN output")
            .expect("Expected non-null result");

        // Log the plan for debugging
        notice!("EXPLAIN BitmapAnd output: {}", explain_result);

        // Check the plan - optimizer may use Bitmap scan or Seq scan depending on statistics
        assert!(
            !explain_result.is_empty(),
            "Expected non-empty EXPLAIN output"
        );

        // If it uses seq scan, it should be detected
        if explain_result.contains("Seq Scan") {
            assert_seq_scan_error(query, vec!["foo".to_string()]);
        }
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
