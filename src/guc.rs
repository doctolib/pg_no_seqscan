// Register the GUC parameters for the extension

use pgrx::{GucContext, GucFlags, GucRegistry, GucSetting, PostgresGucEnum};
use std::ffi::CStr;

#[derive(PostgresGucEnum, Clone, Copy, PartialEq, Debug)]
pub enum DetectionLevelEnum {
    Off,
    Warn,
    Error,
}

pub static PG_NO_SEQSCAN_LEVEL: GucSetting<DetectionLevelEnum> =
    GucSetting::<DetectionLevelEnum>::new(DetectionLevelEnum::Error);

pub static PG_NO_SEQSCAN_CHECK_DATABASES: GucSetting<Option<&'static CStr>> =
    GucSetting::<Option<&'static CStr>>::new(Some(c""));

pub static PG_NO_SEQSCAN_CHECK_SCHEMAS: GucSetting<Option<&'static CStr>> =
    GucSetting::<Option<&'static CStr>>::new(Some(c"public"));

pub static PG_NO_SEQSCAN_IGNORE_USERS: GucSetting<Option<&'static CStr>> =
    GucSetting::<Option<&'static CStr>>::new(Some(c""));

pub static PG_NO_SEQSCAN_IGNORE_TABLES: GucSetting<Option<&'static CStr>> =
    GucSetting::<Option<&'static CStr>>::new(Some(c""));

pub static PG_NO_SEQSCAN_CHECK_TABLES: GucSetting<Option<&'static CStr>> =
    GucSetting::<Option<&'static CStr>>::new(Some(c""));

pub fn register_gucs() {
    GucRegistry::define_enum_guc(
        "pg_no_seqscan.level",
        "Detection level for sequential scans",
        "Error: query failed on seqscan - Warn: a notice is displayed on seqscan - Off: detection skipped",
        &PG_NO_SEQSCAN_LEVEL,
        GucContext::Userset,
        GucFlags::default(),
    );

    GucRegistry::define_string_guc(
        "pg_no_seqscan.check_databases",
        "List of databases to check seqscan for, comma separated",
        "If empty, all databases will be checked",
        &PG_NO_SEQSCAN_CHECK_DATABASES,
        GucContext::Suset,
        GucFlags::SUPERUSER_ONLY,
    );

    GucRegistry::define_string_guc(
        "pg_no_seqscan.check_schemas",
        "List of schemas to check seqscan for, comma separated",
        "",
        &PG_NO_SEQSCAN_CHECK_SCHEMAS,
        GucContext::Suset,
        GucFlags::SUPERUSER_ONLY,
    );

    GucRegistry::define_string_guc(
        "pg_no_seqscan.check_tables",
        "List of tables to check seqscan for, comma separated",
        "If empty, pg_no_seqscan will check all tables",
        &PG_NO_SEQSCAN_CHECK_TABLES,
        GucContext::Suset,
        GucFlags::SUPERUSER_ONLY,
    );

    GucRegistry::define_string_guc(
        "pg_no_seqscan.ignore_users",
        "List of users to ignore, comma separated",
        "",
        &PG_NO_SEQSCAN_IGNORE_USERS,
        GucContext::Suset,
        GucFlags::SUPERUSER_ONLY,
    );

    GucRegistry::define_string_guc(
        "pg_no_seqscan.ignore_tables",
        "List of tables to ignore, comma separated",
        "This setting is ignored if some tables are declared in `check_tables`",
        &PG_NO_SEQSCAN_IGNORE_TABLES,
        GucContext::Suset,
        GucFlags::SUPERUSER_ONLY,
    );
}
