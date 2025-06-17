use crate::guc;
use pgrx::pg_sys::{CmdType, List, NodeTag::T_SeqScan, Oid, Plan, QueryDesc, SeqScan};
use pgrx::{error, notice, pg_sys, PgBox, PgRelation};
#[allow(deprecated)]
use pgrx::{register_hook, HookResult, PgHooks};
use regex::Regex;

use crate::guc::DetectionLevelEnum;
use crate::helpers::{
    current_db_name, current_username, extract_comma_separated_setting, resolve_namespace_name,
    resolve_table_name, scanned_table,
};
use std::ffi::CStr;
#[derive(Clone)]
pub struct NoSeqscanHooks {
    pub is_explain_stmt: bool,
    pub tables_in_seqscans: Vec<String>,
}

impl NoSeqscanHooks {
    fn check_query(&mut self, query_desc: &PgBox<QueryDesc>) {
        // See PlannedStmt documentation: https://github.com/postgres/postgres/blob/master/src/include/nodes/plannodes.h#L46
        let query_string = self.get_query_string(query_desc);

        let plannedstmt_ref = unsafe { query_desc.plannedstmt.as_ref() };
        if plannedstmt_ref.is_none() {
            return;
        }

        let ps = plannedstmt_ref.unwrap();
        self.check_plan_recursively(ps.planTree, ps.rtable);

        if !self.tables_in_seqscans.is_empty() && !self.is_ignored_query_for_comment(&query_string)
        {
            self.report_seqscan(&query_string);
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

    fn report_seqscan(&self, query_string: &str) {
        let message = format!(
            "A 'Sequential Scan' on {} has been detected.
  - Run an EXPLAIN on your query to check the query plan.
  - Make sure the query is compatible with the existing indexes.

Query: {}
",
            self.tables_in_seqscans.join(","),
            query_string
        );
        match guc::PG_NO_SEQSCAN_LEVEL.get() {
            DetectionLevelEnum::Warn => notice!("{message}"),
            DetectionLevelEnum::Error => error!("{message}"),
            DetectionLevelEnum::Off => unreachable!(),
        }
    }

    fn get_query_string(&self, query_desc: &PgBox<QueryDesc>) -> String {
        unsafe { CStr::from_ptr(query_desc.sourceText) }
            .to_str()
            .unwrap()
            .to_string()
            .to_lowercase()
    }

    fn is_ignored_query_for_comment(&mut self, query_string: &str) -> bool {
        let re = Regex::new(r"/\*.*pg_no_seqscan_skip.*\*/").unwrap();
        re.is_match(query_string)
    }

    fn is_ignored_user(&mut self, current_user: String) -> bool {
        match guc::PG_NO_SEQSCAN_IGNORE_USERS.get() {
            Some(ignore_users_setting) => extract_comma_separated_setting(ignore_users_setting)
                .any(|ignore_user| current_user == ignore_user),
            None => unreachable!(),
        }
    }

    fn is_checked_database(&mut self, database: String) -> bool {
        match guc::PG_NO_SEQSCAN_CHECK_DATABASES.get() {
            Some(check_databases_setting) => {
                check_databases_setting.is_empty()
                    || extract_comma_separated_setting(check_databases_setting)
                        .any(|check_database| database == check_database)
            }
            None => unreachable!(),
        }
    }

    fn is_checked_schema(&mut self, schema: String) -> bool {
        match guc::PG_NO_SEQSCAN_CHECK_SCHEMAS.get() {
            Some(check_schemas_setting) => {
                check_schemas_setting.is_empty()
                    || extract_comma_separated_setting(check_schemas_setting)
                        .any(|check_schema| schema == check_schema)
            }
            None => unreachable!(),
        }
    }

    fn check_tables_options_is_set(&mut self) -> bool {
        guc::PG_NO_SEQSCAN_CHECK_TABLES
            .get()
            .is_some_and(|tables| !tables.is_empty())
    }

    fn is_checked_table(&mut self, table_name: String) -> bool {
        match guc::PG_NO_SEQSCAN_CHECK_TABLES.get() {
            Some(check_tables_setting) => {
                check_tables_setting.is_empty()
                    || extract_comma_separated_setting(check_tables_setting)
                        .any(|check_table| table_name == check_table)
            }
            None => unreachable!(),
        }
    }

    fn is_ignored_table(&mut self, table_name: String) -> bool {
        match guc::PG_NO_SEQSCAN_IGNORE_TABLES.get() {
            Some(ignore_tables_setting) => extract_comma_separated_setting(ignore_tables_setting)
                .any(|ignore_table| table_name == ignore_table),
            None => unreachable!(),
        }
    }

    unsafe fn check_current_node(&mut self, node: *mut Plan, rtables: *mut List) {
        if node.as_ref().map(|plan_ref| plan_ref.type_).unwrap() != T_SeqScan {
            return;
        }

        let seq_scan: &mut SeqScan = &mut *(node as *mut SeqScan);
        #[cfg(not(any(feature = "pg13", feature = "pg14")))]
        let table_oid = scanned_table(seq_scan.scan.scanrelid, rtables).unwrap();
        #[cfg(any(feature = "pg13", feature = "pg14"))]
        let table_oid = scanned_table(seq_scan.scanrelid, rtables).unwrap();

        if self.is_sequence(table_oid) {
            return;
        }

        let current_db_name = current_db_name();
        if !self.is_checked_database(current_db_name) {
            return;
        }

        let schema = resolve_namespace_name(table_oid).unwrap();
        if !self.is_checked_schema(schema) {
            return;
        }

        let table_name = resolve_table_name(table_oid);
        let table_name = table_name.unwrap();

        if !self.is_checked_table(table_name.clone()) {
            return;
        }

        if !self.check_tables_options_is_set() && self.is_ignored_table(table_name.clone()) {
            return;
        }

        self.tables_in_seqscans.push(table_name.clone());
    }

    fn is_sequence(&self, relation_oid: Oid) -> bool {
        unsafe {
            let relation = PgRelation::open(relation_oid);
            (*relation.rd_rel).relkind == (pg_sys::RELKIND_SEQUENCE as i8)
        }
    }
}

#[allow(deprecated)]
impl PgHooks for NoSeqscanHooks {
    fn process_utility_hook(
        &mut self,
        mut pstmt: PgBox<pg_sys::PlannedStmt>,
        query_string: &core::ffi::CStr,
        read_only_tree: Option<bool>,
        context: pg_sys::ProcessUtilityContext::Type,
        params: PgBox<pg_sys::ParamListInfoData>,
        query_env: PgBox<pg_sys::QueryEnvironment>,
        dest: PgBox<pg_sys::DestReceiver>,
        completion_tag: *mut pg_sys::QueryCompletion,
        prev_hook: fn(
            pstmt: PgBox<pg_sys::PlannedStmt>,
            query_string: &core::ffi::CStr,
            read_only_tree: Option<bool>,
            context: pg_sys::ProcessUtilityContext::Type,
            params: PgBox<pg_sys::ParamListInfoData>,
            query_env: PgBox<pg_sys::QueryEnvironment>,
            dest: PgBox<pg_sys::DestReceiver>,
            completion_tag: *mut pg_sys::QueryCompletion,
        ) -> HookResult<()>,
    ) -> HookResult<()> {
        if guc::PG_NO_SEQSCAN_LEVEL.get() != DetectionLevelEnum::Off {
            let node: &mut pg_sys::Node = unsafe { &mut *(pstmt.utilityStmt) };
            let is_explain_stmt = node.type_ == pg_sys::NodeTag::T_ExplainStmt;
            if is_explain_stmt {
                unsafe {
                    HOOK_OPTION = Some(NoSeqscanHooks {
                        is_explain_stmt,
                        tables_in_seqscans: Vec::new(),
                    });
                };
            }
        }
        prev_hook(
            pstmt,
            query_string,
            read_only_tree,
            context,
            params,
            query_env,
            dest,
            completion_tag,
        )
    }

    #[allow(static_mut_refs)]
    fn executor_start(
        &mut self,
        query_desc: PgBox<QueryDesc>,
        eflags: i32,
        prev_hook: fn(query_desc: PgBox<QueryDesc>, eflags: i32) -> HookResult<()>,
    ) -> HookResult<()> {
        if guc::PG_NO_SEQSCAN_LEVEL.get() != DetectionLevelEnum::Off {
            let is_explain_stmt = unsafe { HOOK_OPTION.as_ref().unwrap().is_explain_stmt };
            // reset hook state
            unsafe {
                HOOK_OPTION = Some(NoSeqscanHooks {
                    is_explain_stmt: false,
                    tables_in_seqscans: Vec::new(),
                });
            }

            match query_desc.operation {
                CmdType::CMD_SELECT
                | CmdType::CMD_UPDATE
                | CmdType::CMD_INSERT
                | CmdType::CMD_DELETE => {
                    if !is_explain_stmt && !self.is_ignored_user(unsafe { current_username() }) {
                        self.check_query(&query_desc);
                    }
                }
                #[cfg(not(any(feature = "pg13", feature = "pg14")))]
                CmdType::CMD_MERGE => {
                    if !is_explain_stmt && !self.is_ignored_user(unsafe { current_username() }) {
                        self.check_query(&query_desc);
                    }
                }
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
        is_explain_stmt: false,
        tables_in_seqscans: Vec::new(),
    });
    register_hook(HOOK_OPTION.as_mut().unwrap())
}
