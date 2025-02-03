use crate::guc;
use pgrx::pg_sys::{CmdType, List, NodeTag::T_SeqScan, Plan, QueryDesc, SeqScan};
use pgrx::{error, notice, PgBox};
#[allow(deprecated)]
use pgrx::{register_hook, HookResult, PgHooks};
use regex::Regex;

use crate::guc::{DetectionLevelEnum, PG_NO_SEQSCAN_LEVEL};
use crate::helpers::{resolve_namespace_name, resolve_table_name, scanned_table};
use std::ffi::CStr;

pub struct NoSeqscanHooks {
    pub tables_in_seqscans: Vec<String>,
}

impl NoSeqscanHooks {
    fn check_query(&mut self, query_desc: &PgBox<QueryDesc>) {
        // See PlannedStmt documentation: https://github.com/postgres/postgres/blob/master/src/include/nodes/plannodes.h#L46
        let query_string = unsafe { CStr::from_ptr(query_desc.sourceText) }
            .to_str()
            .unwrap()
            .to_string()
            .to_lowercase();

        if self.ignore_query_for_explain(&query_string) {
            return;
        }

        let plannedstmt_ref = unsafe { query_desc.plannedstmt.as_ref() };
        if plannedstmt_ref.is_none() {
            return;
        }

        let ps = plannedstmt_ref.unwrap();
        self.check_plan_recursively(ps.planTree, ps.rtable);

        if !self.tables_in_seqscans.is_empty() {
            if self.ignore_query_for_comment(&query_string) {
                return;
            }

            let message = format!(
                "A 'Sequential Scan' on {} has been detected.
  - Run an EXPLAIN on your query to check the query plan.
  - Make sure the query is compatible with the existing indexes.

Query: {}
",
                self.tables_in_seqscans.join(","),
                query_string
            );
            match PG_NO_SEQSCAN_LEVEL.get() {
                DetectionLevelEnum::Warn => notice!("{message}"),
                DetectionLevelEnum::Error => error!("{message}"),
                DetectionLevelEnum::Off => unreachable!(),
            }
        }
    }

    fn check_plan_recursively(&mut self, plan: *mut Plan, rtables: *mut List) {
        unsafe {
            if let Some(node) = plan.as_ref() {
                self.check_current_node(plan, rtables);

                self.check_plan_recursively(node.lefttree, rtables);
                self.check_plan_recursively(node.righttree, rtables);
            }
        }
    }

    fn ignore_query_for_explain(&mut self, query_string: &str) -> bool {
        let query_first_word = query_string.split_whitespace().next().unwrap_or("");

        return query_first_word == "explain";
    }

    fn ignore_query_for_comment(&mut self, query_string: &str) -> bool {
        let re = Regex::new(r"/\*\s*pg_no_seqscan_skip\s*\*/").unwrap();

        return re.is_match(&query_string);
    }

    unsafe fn check_current_node(&mut self, node: *mut Plan, rtables: *mut List) {
        if node.as_ref().map(|plan_ref| plan_ref.type_).unwrap() != T_SeqScan {
            return;
        }

        let seq_scan: &mut SeqScan = &mut *(node as *mut SeqScan);
        let table_oid = scanned_table(seq_scan.scan.scanrelid, rtables).unwrap();
        let schema = resolve_namespace_name(table_oid).unwrap();

        let ignored_schemas = guc::PG_NO_SEQSCAN_IGNORED_SCHEMAS
            .get()
            .unwrap()
            .to_str()
            .expect("Ignored schema should be valid");
        if ignored_schemas
            .split(',')
            .any(|ignored_schema| schema == ignored_schema)
        {
            return;
        }

        let table_name = resolve_table_name(table_oid);
        let table_name = table_name.unwrap();
        self.tables_in_seqscans.push(table_name.clone());
    }
}

#[allow(deprecated)]
impl PgHooks for NoSeqscanHooks {
    fn executor_start(
        &mut self,
        query_desc: PgBox<QueryDesc>,
        eflags: i32,
        prev_hook: fn(query_desc: PgBox<QueryDesc>, eflags: i32) -> HookResult<()>,
    ) -> HookResult<()> {
        if PG_NO_SEQSCAN_LEVEL.get() != DetectionLevelEnum::Off {
            unsafe {
                HOOK_OPTION = Some(NoSeqscanHooks {
                    tables_in_seqscans: Vec::new(),
                })
            };
            match query_desc.operation {
                CmdType::CMD_SELECT
                | CmdType::CMD_UPDATE
                | CmdType::CMD_INSERT
                | CmdType::CMD_DELETE
                | CmdType::CMD_MERGE => self.check_query(&query_desc),
                _ => {}
            }
        }
        prev_hook(query_desc, eflags)
    }
}

pub static mut HOOK_OPTION: Option<NoSeqscanHooks> = None;

#[allow(deprecated, static_mut_refs)]
pub unsafe fn init_hooks() {
    HOOK_OPTION = Some(NoSeqscanHooks {
        tables_in_seqscans: Vec::new(),
    });
    register_hook(HOOK_OPTION.as_mut().unwrap())
}
