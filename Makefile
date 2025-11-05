PG_CONFIG   ?= $(shell which pg_config)
DISTNAME     = $(shell perl -nE '/^name\s*=\s*"([^"]+)/ && do { say $$1; exit }' Cargo.toml)
DISTVERSION  = $(shell perl -nE '/^version\s*=\s*"([^"]+)/ && do { say $$1; exit }' Cargo.toml)
PGRXV        = $(shell perl -nE '/^pgrx\s+=\s"=?([^"]+)/ && do { say $$1; exit }' Cargo.toml)
PGV          = $(shell perl -E 'shift =~ /(\d+)/ && say $$1' "$(shell $(PG_CONFIG) --version)")
PGOPTIONS="$PGOPTIONS  -fs                "
EXTRA_CLEAN  = META.json $(DISTNAME)-$(DISTVERSION).zip target
TESTS        = $(wildcard tests/pg_regress/sql/*.sql)
REGRESS      = setup $(filter-out setup,$(sort $(patsubst tests/pg_regress/sql/%.sql,%,$(TESTS))))
REGRESS_OPTS = --inputdir=tests/pg_regress --outputdir=target/installcheck

PGXS := $(shell $(PG_CONFIG) --pgxs)
include $(PGXS)
