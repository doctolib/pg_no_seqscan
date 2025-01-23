In order to use this bgworker with pgrx, you'll need to edit the proper postgresql.conf file in "${PGRX_HOME}/data-$PGVER/postgresql.conf" and add this line to the end:

    shared_preload_libraries = 'pg_no_seqscan.so'
    logging_collector = on
    log_filename = 'postgresql.log'

Pg_no_seqscan must be initialized in the extension's _PG_init() function, and can only be started if loaded through the shared_preload_libraries configuration setting.


tail -f ~/.pgrx/data-15/log/postgresql.log
