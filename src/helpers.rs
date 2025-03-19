use pgrx::pg_sys::{get_namespace_name, get_rel_name, get_rel_namespace, rt_fetch, List, Oid};
use std::ffi::{c_char, CStr};

pub fn extract_comma_separated_setting(comma_separated_string: &CStr) -> std::str::Split<'_, char> {
    comma_separated_string.to_str().unwrap().split(',')
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
