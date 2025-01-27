use pgrx::notice;
use pgrx::pg_sys::CmdType;
use pgrx::pg_sys::{get_rel_name, rt_fetch, List, NodeTag::T_SeqScan, Plan, SeqScan};
#[allow(deprecated)]
use pgrx::{pg_sys, register_hook, HookResult, PgBox, PgHooks};
use std::ffi::CStr;

struct NoSeqscanHooks;

unsafe fn resolve_table_name(rtables: *mut List, scanrelid: u32) -> String {
    let relname = get_rel_name(rt_fetch(scanrelid, rtables).as_ref().unwrap().relid);
    let c_str: &CStr = unsafe { CStr::from_ptr(relname) };
    c_str.to_str().unwrap().to_string()
}

fn notice_on_seq_scans(plan: *mut Plan, rtables: *mut List, query_string: &String) {
    unsafe {
        plan.as_ref().map(|plan_ref| {
            if plan_ref.type_ == T_SeqScan {
                let seq_scan: &mut SeqScan = &mut *(plan as *mut SeqScan);
                notice!(
                    "{:?} on table: '{}' - query: '{}'",
                    plan_ref.type_,
                    resolve_table_name(rtables, seq_scan.scan.scanrelid),
                    query_string
                );
            } else {
                // See Plan documentation: https://github.com/postgres/postgres/blob/master/src/include/nodes/plannodes.h#L119
                notice_on_seq_scans(plan_ref.lefttree, rtables, query_string);
                notice_on_seq_scans(plan_ref.righttree, rtables, query_string);
            }
        });
    }
}

#[allow(deprecated)]
impl PgHooks for NoSeqscanHooks {
    fn executor_start(
        &mut self,
        query_desc: PgBox<pg_sys::QueryDesc>,
        eflags: i32,
        prev_hook: fn(query_desc: PgBox<pg_sys::QueryDesc>, eflags: i32) -> HookResult<()>,
    ) -> HookResult<()> {
        // See PlannedStmt documentation: https://github.com/postgres/postgres/blob/master/src/include/nodes/plannodes.h#L46
        let query_string = unsafe { CStr::from_ptr(query_desc.sourceText) }
            .to_str()
            .unwrap()
            .to_string()
            .to_lowercase();

        if query_desc.operation == CmdType::CMD_SELECT {
            let query_first_word = query_string.split_whitespace().next().unwrap_or("");

            if query_first_word != "explain" {
                unsafe {
                    query_desc
                        .plannedstmt
                        .as_ref()
                        .map(|ps| notice_on_seq_scans(ps.planTree, ps.rtable, &query_string));
                }
            }
        }

        prev_hook(query_desc, eflags)
    }
}

static mut HOOKS: NoSeqscanHooks = NoSeqscanHooks;

#[allow(deprecated, static_mut_refs)]
pub unsafe fn init_hooks() {
    register_hook(&mut HOOKS)
}
