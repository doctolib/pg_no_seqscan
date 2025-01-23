use pgrx::pg_sys::NodeTag::T_SeqScan;
use pgrx::pg_sys::Plan;
#[allow(deprecated)]
use pgrx::{pg_sys, register_hook, HookResult, PgBox, PgHooks};
use pgrx::notice;
use regex::Regex;

struct NoSeqscanHooks;

fn notice_on_seq_scans(plan: &Plan) {
    // See Plan documentation: https://github.com/postgres/postgres/blob/master/src/include/nodes/plannodes.h#L119
    let (left_ref, right_ref, _target_list) = unsafe {
        (
            plan.lefttree.as_ref(),
            plan.righttree.as_ref(),
            plan.targetlist.as_ref(),
        )
    };

    match left_ref {
        Some(left_plan) => {
            notice_on_seq_scans(left_plan);
        }
        None => {}
    }

    match right_ref {
        Some(right_plan) => {
            notice_on_seq_scans(right_plan);
        }
        None => {}
    }

    if plan.type_ != T_SeqScan {
        return;
    }

    let targetlist = unsafe { plan.targetlist.as_ref() };
    let target_string = targetlist.map_or(String::from(""), |t| t.to_string());

    // TODO find oid with an other strategy
    // Parsing something like
    // "({TARGETENTRY :expr {VAR :varno 1 :varattno 1 :vartype 23 :vartypmod -1 :varcollid 0 :varlevelsup 0 :varnosyn 1 :varattnosyn 1 :location 7} :resno 1 :resname id :ressortgroupref 0 :resorigtbl 16398 :resorigcol 1 :resjunk false})"
    let re = Regex::new(r":resorigtbl (?<resorigtabl>\d+) :").unwrap();
    let Some(result) = re.captures(&target_string) else {
        return;
    };

    notice!(
        "{:?} on table with oid: {}",
        plan.type_,
        &result["resorigtabl"],
    );

    /*
    notice!(
        "[#{:?}]({:?}) something: {:?}",
        plan.plan_node_id,
        plan.type_,
        targetlist.map_or(String::from(""), |target_list| format!(
            "{:?} {:?}",
            unsafe { target_list.elements.as_ref().unwrap().oid_value },
            // target_list.type_,
            // target_list.length,
            unsafe { (*pgrx::pg_sys::list_head(target_list)).oid_value },
            // unsafe { (*target_list.elements.wrapping_add(0)).oid_value }
            //unsafe { (*pgrx::pg_sys::list_head(target_list) as TargetEntry).resorigtbl },
        ))
    );
    */
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
            match query_desc
                .plannedstmt
                .as_ref()
                .and_then(|ps| ps.planTree.as_ref())
            {
                Some(plan_ref) => notice_on_seq_scans(plan_ref),
                _ => {}
            }
        };
        /*notice!(
            "            executor_start HOOK, {:?}",
            format!("query_desc: {}", unsafe { *query_desc.plannedstmt })
        );*/
        prev_hook(query_desc, eflags)
    }
}

static mut HOOKS: NoSeqscanHooks = NoSeqscanHooks;

#[allow(deprecated, static_mut_refs)]
pub unsafe fn init_hooks() {
    register_hook(&mut HOOKS)
}
