use pgrx::pg_sys::{
    GetUserId, GetUserNameFromId, List, MyDatabaseId, Oid, get_database_name, get_namespace_name,
    get_rel_name, get_rel_namespace, rt_fetch,
};
use pgrx::{PgRelation, Spi};
use std::ffi::{CStr, c_char};

pub fn comma_separated_list_contains(comma_separated_string: &CStr, value: &str) -> bool {
    comma_separated_string
        .to_str()
        .unwrap_or_default()
        .split(',')
        .any(|s| s.trim() == value)
}

pub fn string_from_ptr(ptr: *const c_char) -> Option<String> {
    unsafe { CStr::from_ptr(ptr).to_str().ok().map(String::from) }
}

fn ptr_to_option_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        string_from_ptr(ptr)
    }
}

pub fn scanned_table(scanrelid: u32, rtables: *mut List) -> Option<Oid> {
    unsafe { rt_fetch(scanrelid, rtables).as_ref().map(|rte| rte.relid) }
}

pub fn resolve_namespace_name(oid: Oid) -> Option<String> {
    ptr_to_option_string(unsafe { get_namespace_name(get_rel_namespace(oid)) })
}

pub fn resolve_table_name(table_oid: Oid) -> Option<String> {
    ptr_to_option_string(unsafe { get_rel_name(table_oid) })
}

pub fn current_db_name() -> String {
    unsafe {
        let db_oid = MyDatabaseId;
        string_from_ptr(get_database_name(db_oid)).expect("Failed to get database name")
    }
}

pub fn current_username() -> String {
    let current_user = unsafe { GetUserNameFromId(GetUserId(), true) };
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
