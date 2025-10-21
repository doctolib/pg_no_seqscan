use pgrx::pg_sys;
use pgrx::pg_sys::{get_namespace_name, get_rel_name, get_rel_namespace, rt_fetch, List, Oid};
use pgrx::{PgRelation, Spi};
use std::ffi::{c_char, CStr, CString};

pub fn extract_comma_separated_setting(comma_separated_string: CString) -> Vec<String> {
    comma_separated_string
        .to_str()
        .unwrap()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect()
}
pub fn comma_separated_list_contains(comma_separated_string: CString, value: String) -> bool {
    extract_comma_separated_setting(comma_separated_string).contains(&value)
}

pub unsafe fn string_from_ptr(ptr: *const c_char) -> Option<String> {
    match CStr::from_ptr(ptr).to_str() {
        Ok(str_value) => Some(str_value.to_string()),
        Err(_) => None,
    }
}

pub unsafe fn scanned_table(scanrelid: u32, rtables: *mut List) -> Option<Oid> {
    rt_fetch(scanrelid, rtables).as_ref().map(|rte| rte.relid)
}

pub unsafe fn resolve_namespace_name(oid: Oid) -> Option<String> {
    let namespace_name = get_namespace_name(get_rel_namespace(oid));
    if namespace_name.is_null() {
        None
    } else {
        string_from_ptr(namespace_name)
    }
}

pub unsafe fn resolve_table_name(table_oid: Oid) -> Option<String> {
    let relname_ptr = get_rel_name(table_oid);
    if relname_ptr.is_null() {
        return None;
    }

    string_from_ptr(relname_ptr)
}

pub unsafe fn current_db_name() -> String {
    let db_oid = pg_sys::MyDatabaseId;
    string_from_ptr(pg_sys::get_database_name(db_oid)).expect("Failed to get database name")
}

pub unsafe fn current_username() -> String {
    let current_user = unsafe { pg_sys::GetUserNameFromId(pg_sys::GetUserId(), true) };
    string_from_ptr(current_user).expect("Failed to get username")
}

pub unsafe fn get_parent_table_oid(table_oid: Oid) -> Option<Oid> {
    if !(*PgRelation::open(table_oid).rd_rel).relispartition {
        return None;
    }

    Spi::get_one::<Oid>(&format!(
        "with recursive inheritance (oid, child_oid) AS (
select {}::regclass, null::oid
union all
select inhparent, oid from inheritance
           left join pg_inherits on inheritance.oid = inhrelid
                 where inheritance.oid is not null
)
select child_oid::regclass as root_partition from inheritance where child_oid IS NOT NULL AND oid is null",
        table_oid
    ))
    .ok()
    .flatten()
}
