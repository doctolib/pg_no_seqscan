use pgrx::pg_sys;
use pgrx::pg_sys::{List, Oid, get_namespace_name, get_rel_name, get_rel_namespace, rt_fetch};
use pgrx::{PgRelation, Spi};
use std::ffi::{CStr, CString, c_char};

pub fn extract_comma_separated_setting(comma_separated_string: CString) -> Vec<String> {
    comma_separated_string
        .to_str()
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect()
}
pub fn comma_separated_list_contains(comma_separated_string: CString, value: String) -> bool {
    extract_comma_separated_setting(comma_separated_string).contains(&value)
}

pub fn string_from_ptr(ptr: *const c_char) -> Option<String> {
    match unsafe { CStr::from_ptr(ptr).to_str() } {
        Ok(str_value) => Some(str_value.to_string()),
        Err(_) => None,
    }
}

pub fn scanned_table(scanrelid: u32, rtables: *mut List) -> Option<Oid> {
    unsafe { rt_fetch(scanrelid, rtables).as_ref().map(|rte| rte.relid) }
}

pub fn resolve_namespace_name(oid: Oid) -> Option<String> {
    let namespace_name = unsafe { get_namespace_name(get_rel_namespace(oid)) };
    if namespace_name.is_null() {
        None
    } else {
        string_from_ptr(namespace_name)
    }
}

pub fn resolve_table_name(table_oid: Oid) -> Option<String> {
    let relname_ptr = unsafe { get_rel_name(table_oid) };
    if relname_ptr.is_null() {
        return None;
    }

    string_from_ptr(relname_ptr)
}

pub fn current_db_name() -> String {
    unsafe {
        let db_oid = pg_sys::MyDatabaseId;
        string_from_ptr(pg_sys::get_database_name(db_oid)).expect("Failed to get database name")
    }
}

pub fn current_username() -> String {
    let current_user = unsafe { pg_sys::GetUserNameFromId(pg_sys::GetUserId(), true) };
    string_from_ptr(current_user).expect("Failed to get username")
}

pub fn get_parent_table_oid(table_oid: Oid) -> Option<Oid> {
    if unsafe { !(*PgRelation::open(table_oid).rd_rel).relispartition } {
        return None;
    }

    Spi::get_one::<Oid>(&format!("
WITH RECURSIVE inheritance (oid, child_oid) AS (
    SELECT {}::regclass, null::oid
    UNION ALL
    SELECT inhparent, oid FROM inheritance
    LEFT JOIN pg_inherits ON inheritance.oid = inhrelid
    WHERE inheritance.oid IS NOT NULL
)
SELECT child_oid::regclass AS root_partition FROM inheritance WHERE child_oid IS NOT NULL AND oid IS NULL",
        table_oid
    ))
    .ok()
    .flatten()
}
