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

pub static PG_NO_SEQSCAN_IGNORED_SCHEMAS: GucSetting<Option<&'static CStr>> =
    GucSetting::<Option<&'static CStr>>::new(Some(c"pg_catalog,information_schema"));

pub static PG_NO_SEQSCAN_IGNORED_USERS: GucSetting<Option<&'static CStr>> =
    GucSetting::<Option<&'static CStr>>::new(None);

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
        "pg_no_seqscan.ignored_schemas",
        "List of schemas to ignore, comma separated",
        "",
        &PG_NO_SEQSCAN_IGNORED_SCHEMAS,
        GucContext::Suset,
        GucFlags::SUPERUSER_ONLY,
    );

    GucRegistry::define_string_guc(
        "pg_no_seqscan.ignored_users",
        "List of users to ignore, comma separated",
        "",
        &PG_NO_SEQSCAN_IGNORED_USERS,
        GucContext::Suset,
        GucFlags::SUPERUSER_ONLY,
    );
}
