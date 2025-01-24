use std::ffi::CStr;
use pgrx::pg_sys::{get_rel_name, rt_fetch, CmdType::CMD_UTILITY, List, NodeTag::T_SeqScan, Plan, SeqScan};
#[allow(deprecated)]
use pgrx::{pg_sys, register_hook, HookResult, PgBox, PgHooks};
use pgrx::notice;

struct NoSeqscanHooks;

unsafe fn resolve_table_name(rtables: *mut List, scanrelid: u32) -> String {
    let relname = get_rel_name(rt_fetch(scanrelid, rtables).as_ref().unwrap().relid);
    let c_str: &CStr = unsafe { CStr::from_ptr(relname) };
    c_str.to_str().unwrap().to_string()
}

fn notice_on_seq_scans(plan: *mut Plan, rtables: *mut List) {
    unsafe {
        plan.as_ref().map(|plan_ref| {
            if plan_ref.type_ == T_SeqScan {
                let seq_scan: &mut SeqScan = &mut *(plan as *mut SeqScan);
                notice!(
                    "{:?} on table: '{}'",
                    plan_ref.type_,
                    resolve_table_name(rtables, seq_scan.scan.scanrelid)
                );
            } else {
                // See Plan documentation: https://github.com/postgres/postgres/blob/master/src/include/nodes/plannodes.h#L119
                notice_on_seq_scans(plan_ref.lefttree, rtables);
                notice_on_seq_scans(plan_ref.righttree, rtables);
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
        unsafe {
            query_desc
                .plannedstmt
                .as_ref()
                .map (|ps| notice_on_seq_scans(ps.planTree, ps.rtable));

        }
        prev_hook(query_desc, eflags)
    }
}

static mut HOOKS: NoSeqscanHooks = NoSeqscanHooks;

#[allow(deprecated, static_mut_refs)]
pub unsafe fn init_hooks() {
    register_hook(&mut HOOKS)
}
