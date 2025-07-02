// Register the GUC parameters for the extension

use pgrx::{GucContext, GucFlags, GucRegistry, GucSetting, PostgresGucEnum};
use std::ffi::CString;

#[derive(PostgresGucEnum, Clone, Copy, PartialEq, Debug)]
pub enum DetectionLevelEnum {
    Off,
    Warn,
    Error,
}

pub static PG_NO_SEQSCAN_LEVEL: GucSetting<DetectionLevelEnum> =
    GucSetting::<DetectionLevelEnum>::new(DetectionLevelEnum::Error);

pub static PG_NO_SEQSCAN_CHECK_DATABASES: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(Some(c""));

pub static PG_NO_SEQSCAN_CHECK_SCHEMAS: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(Some(c"public"));

pub static PG_NO_SEQSCAN_IGNORE_USERS: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(Some(c""));

pub static PG_NO_SEQSCAN_IGNORE_TABLES: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(Some(c""));

pub static PG_NO_SEQSCAN_CHECK_TABLES: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(Some(c""));

pub fn register_gucs() {
    GucRegistry::define_enum_guc(
        c"pg_no_seqscan.level",
        c"Detection level for sequential scans",
        c"Error: query failed on seqscan - Warn: a notice is displayed on seqscan - Off: detection skipped",
        &PG_NO_SEQSCAN_LEVEL,
        GucContext::Userset,
        GucFlags::default(),
    );

    GucRegistry::define_string_guc(
        c"pg_no_seqscan.check_databases",
        c"Databases to check seqscan for, comma separated",
        c"If empty, all databases will be checked",
        &PG_NO_SEQSCAN_CHECK_DATABASES,
        GucContext::Suset,
        GucFlags::SUPERUSER_ONLY,
    );

    GucRegistry::define_string_guc(
        c"pg_no_seqscan.check_schemas",
        c"Schemas to check seqscan for, comma separated",
        c"If empty, all schemas will be checked",
        &PG_NO_SEQSCAN_CHECK_SCHEMAS,
        GucContext::Suset,
        GucFlags::SUPERUSER_ONLY,
    );

    GucRegistry::define_string_guc(
        c"pg_no_seqscan.check_tables",
        c"Tables to check seqscan for, comma separated",
        c"If empty, all tables will be checked",
        &PG_NO_SEQSCAN_CHECK_TABLES,
        GucContext::Suset,
        GucFlags::SUPERUSER_ONLY,
    );

    GucRegistry::define_string_guc(
        c"pg_no_seqscan.ignore_users",
        c"Users to ignore, comma separated",
        c"",
        &PG_NO_SEQSCAN_IGNORE_USERS,
        GucContext::Suset,
        GucFlags::SUPERUSER_ONLY,
    );

    GucRegistry::define_string_guc(
        c"pg_no_seqscan.ignore_tables",
        c"Tables to ignore, comma separated",
        c"This setting is ignored if some tables are declared in `check_tables`",
        &PG_NO_SEQSCAN_IGNORE_TABLES,
        GucContext::Suset,
        GucFlags::SUPERUSER_ONLY,
    );
}
