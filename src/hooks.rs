use crate::guc::{DetectionLevelEnum,
                 PG_NO_SEQSCAN_CHECK_DATABASES,PG_NO_SEQSCAN_IGNORE_TABLES,
                 PG_NO_SEQSCAN_CHECK_SCHEMAS, PG_NO_SEQSCAN_CHECK_TABLES, PG_NO_SEQSCAN_IGNORE_USERS, PG_NO_SEQSCAN_LEVEL};
use crate::helpers::{
    comma_separated_list_contains, current_db_name, current_username, get_parent_table_oid,
    resolve_namespace_name, resolve_table_name, scanned_table,
};
use pgrx::pg_sys::{
    ExecutorStart_hook_type,
    ffi::pg_guard_ffi_boundary,
    Append, CmdType, ExplainPrintPlan, List, NewExplainState, NodeTag::{T_Append, T_SeqScan, T_SubqueryScan},
    Oid, Plan, QueryDesc, SeqScan, SubqueryScan, EXEC_FLAG_EXPLAIN_ONLY,
};
use pgrx::{error, notice, pg_guard, pg_sys, PgBox, PgRelation};
use regex::{Error, Regex};
use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::LazyLock;

#[derive(Clone)]
pub struct NoSeqscanHooks {
    pub tables_in_seqscans: HashSet<String>,
}

static SKIP_COMMENT_RE: LazyLock<Result<Regex, Error>> = LazyLock::new(|| Regex::new(r"/\*.*pg_no_seqscan_skip.*\*/"));
fn is_ignored_query_for_comment(query_string: &str) -> bool {
    SKIP_COMMENT_RE
        .as_ref()
        .ok()
        .map(|re| re.is_match(query_string))
        .unwrap_or(false)
}

impl NoSeqscanHooks {
    fn check_query(&mut self, query_desc: &PgBox<QueryDesc>) {
        // See PlannedStmt documentation: https://github.com/postgres/postgres/blob/master/src/include/nodes/plannodes.h#L46
        let query_string = self.get_query_string(query_desc);

        let plannedstmt_ref = unsafe { query_desc.plannedstmt.as_ref() };
        if let Some(ps) = plannedstmt_ref {
            self.check_plan_recursively(ps.planTree, ps.rtable);

            // Queries with CTEs generate subplans
            if !ps.subplans.is_null() {
                self.check_plan_list(ps.subplans, ps.rtable);
            }

            if !self.tables_in_seqscans.is_empty()
                && !is_ignored_query_for_comment(&query_string)
            {
                unsafe {
                    let explain_state = NewExplainState();
                    (*explain_state).costs = false;
                    ExplainPrintPlan(explain_state, query_desc.as_ptr());
                    let explain_output = std::ffi::CStr::from_ptr((*(*explain_state).str_).data)
                        .to_str()
                        .unwrap_or("Invalid UTF-8 in query plan");
                    self.report_seqscan(&query_string, explain_output);
                }
            }
        }
    }

    fn check_plan_recursively(&mut self, plan: *mut Plan, rtables: *mut List) {
        unsafe {
            let Some(node) = plan.as_ref() else { return };

            self.check_current_node(plan, rtables);

            match node.type_ {
                T_Append => self.check_plan_list((*(plan as *mut Append)).appendplans, rtables),
                T_SubqueryScan => {
                    if let Some(plan) = (plan as *mut SubqueryScan).as_ref() {
                        self.check_plan_recursively(plan.subplan, rtables)
                    }
                }
                _ => {}
            }

            self.check_plan_recursively(node.lefttree, rtables);
            self.check_plan_recursively(node.righttree, rtables);
        }
    }

    fn check_plan_list(&mut self, subplans: *mut List, rtables: *mut List) {
        unsafe {
            for i in 0..(*subplans).length {
                if let Some(cell) = pg_sys::list_nth_cell(subplans, i).as_ref() {
                    self.check_plan_recursively(cell.ptr_value as *mut Plan, rtables);
                }
            }
        }
    }

    fn report_seqscan(&self, query_string: &str, explain_output: &str) {
        let mut tables: Vec<_> = self.tables_in_seqscans.iter().cloned().collect();
        tables.sort();
        let message = format!(
            "A 'Sequential Scan' has been detected. Make sure the query is compatible with the existing indexes.
  - Tables involved: {}
  - Query: {}
  - Query plan:

  {}
",
            tables.join("\n"),
            query_string,
            explain_output,
        );
        match PG_NO_SEQSCAN_LEVEL.get() {
            DetectionLevelEnum::Warn => notice!("{message}"),
            DetectionLevelEnum::Error => error!("{message}"),
            DetectionLevelEnum::Off => unreachable!(),
        }
    }

    fn get_query_string(&self, query_desc: &PgBox<QueryDesc>) -> String {
        unsafe { CStr::from_ptr(query_desc.sourceText) }
            .to_str()
            .expect("Invalid UTF-8 in query string")
            .to_string()
    }

    fn is_ignored_user(&self, current_user: &str) -> bool {
        PG_NO_SEQSCAN_IGNORE_USERS
            .get()
            .map(|ignore_users_setting| {
                comma_separated_list_contains(ignore_users_setting, current_user)
            })
            .unwrap_or(false)
    }

    fn is_checked_database(&self, database: &str) -> bool {
        PG_NO_SEQSCAN_CHECK_DATABASES
            .get()
            .map(|check_databases_setting| {
                check_databases_setting.is_empty()
                    || comma_separated_list_contains(check_databases_setting, database)
            })
            .unwrap_or(true)
    }

    fn is_checked_schema(&self, schema: &str) -> bool {
        PG_NO_SEQSCAN_CHECK_SCHEMAS
            .get()
            .map(|check_schemas_setting| {
                check_schemas_setting.is_empty()
                    || comma_separated_list_contains(check_schemas_setting, schema)
            })
            .unwrap_or(true)
    }

    fn check_tables_options_is_set(&self) -> bool {
        PG_NO_SEQSCAN_CHECK_TABLES
            .get()
            .is_some_and(|tables| !tables.is_empty())
    }

    fn is_checked_table(&self, table_name: &str) -> bool {
        PG_NO_SEQSCAN_CHECK_TABLES
            .get()
            .map(|check_tables_setting| {
                check_tables_setting.is_empty()
                    || comma_separated_list_contains(check_tables_setting, table_name)
            })
            .unwrap_or(true)
    }

    fn is_ignored_table(&self, table_name: &str) -> bool {
        PG_NO_SEQSCAN_IGNORE_TABLES
            .get()
            .map(|ignore_tables_setting| {
                comma_separated_list_contains(ignore_tables_setting, table_name)
            })
            .unwrap_or(false)
    }

    fn check_current_node(&mut self, node: *mut Plan, rtables: *mut List) {
        unsafe {
            if node.as_ref().map(|plan_ref| plan_ref.type_) != Some(T_SeqScan) {
                return;
            }

            let seq_scan: &mut SeqScan = &mut *(node as *mut SeqScan);
            #[cfg(not(feature = "pg14"))]
            let table_oid = scanned_table(seq_scan.scan.scanrelid, rtables)
                .expect("Failed to get scanned table OID");
            #[cfg(feature = "pg14")]
            let table_oid = scanned_table(seq_scan.scanrelid, rtables)
                .expect("Failed to get scanned table OID");

            if self.is_sequence(table_oid) {
                return;
            }

            let current_db_name = current_db_name();
            if !self.is_checked_database(&current_db_name) {
                return;
            }

            let schema = resolve_namespace_name(table_oid).expect("Failed to resolve schema name");
            if !self.is_checked_schema(&schema) {
                return;
            }

            // Check if this table is a partition, and if so, use the parent table name
            let report_table_name = if let Some(parent_oid) = get_parent_table_oid(table_oid) {
                resolve_table_name(parent_oid).expect("Failed to resolve parent table name")
            } else {
                resolve_table_name(table_oid).expect("Failed to resolve table name")
            };

            if !self.is_checked_table(&report_table_name) {
                return;
            }

            if !self.check_tables_options_is_set()
                && self.is_ignored_table(&report_table_name)
            {
                return;
            }

            self.tables_in_seqscans.insert(report_table_name.clone());
        }
    }

    fn is_sequence(&self, relation_oid: Oid) -> bool {
        unsafe {
            let relation = PgRelation::open(relation_oid);
            (*relation.rd_rel).relkind == (pg_sys::RELKIND_SEQUENCE as c_char)
        }
    }

    fn check_query_plan(&mut self, query_desc: PgBox<QueryDesc>) {
        // reset hook state
        HOOK_OPTION.with(|c| {
            *c.borrow_mut() = Some(NoSeqscanHooks {
                tables_in_seqscans: HashSet::new(),
            });
        });

        match query_desc.operation {
            CmdType::CMD_SELECT
            | CmdType::CMD_UPDATE
            | CmdType::CMD_INSERT
            | CmdType::CMD_DELETE => {
                if !self.is_ignored_user(&current_username()) {
                    self.check_query(&query_desc);
                }
            }
            #[cfg(not(any(feature = "pg14")))]
            CmdType::CMD_MERGE => {
                if !self.is_ignored_user(&current_username()) {
                    self.check_query(&query_desc);
                }
            }
            _ => {}
        }
    }
}

thread_local! {
    static HOOK_OPTION: RefCell<Option<NoSeqscanHooks>> = const{ RefCell::new(None) };
    static PREV_EXECUTOR_START: RefCell<ExecutorStart_hook_type> = const{ RefCell::new(None) };
}

pub fn init_hooks() {
    #[pg_guard]
    extern "C-unwind" fn executor_start_hook(
        query_desc: *mut QueryDesc,
        eflags: ::core::ffi::c_int,
    ) {
        unsafe {
            pg_sys::standard_ExecutorStart(query_desc, eflags);
            let query_desc_box = PgBox::from_pg(query_desc);
            let is_explain_only = (eflags & EXEC_FLAG_EXPLAIN_ONLY as i32) != 0;
            // In postgres code if es->analyze then at least INSTRUMENT_ROWS is set
            let has_any_instrumentation = query_desc_box.instrument_options != 0;
            // Skip if it's EXPLAIN (with or without ANALYZE)
            let is_explain_context = is_explain_only || has_any_instrumentation;

            if PG_NO_SEQSCAN_LEVEL.get() != DetectionLevelEnum::Off && !is_explain_context
            {
                if let Some(mut hook_option) = HOOK_OPTION.with(|c| c.borrow().clone()) {
                    hook_option.check_query_plan(PgBox::from_pg(query_desc));
                }
            }
            if let Some(prev_hook) = PREV_EXECUTOR_START.with(|c| *c.borrow()) {
                pg_guard_ffi_boundary(|| prev_hook(query_desc, eflags));
            }
        }
    }

    HOOK_OPTION.with(|c| {
        *c.borrow_mut() = Some(NoSeqscanHooks {
            tables_in_seqscans: HashSet::new(),
        });
    });

    PREV_EXECUTOR_START.with(|c| unsafe {
        pg_sys::ExecutorStart_hook = Some(executor_start_hook);
        *c.borrow_mut() = pg_sys::ExecutorStart_hook;
    });
}
