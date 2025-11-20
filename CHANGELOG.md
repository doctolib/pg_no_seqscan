# CHANGELOG

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Version 0.1.0

### Added
- Detect seqscan in partitioned tables.
- Support PostgreSQL 18.
- Display query plan in the seq scan error message.

### Fixed
- Don't repeat table names when multiple parts of the plan do seqscan on it.
- Order table names alphabetically in error messages.

### Changed
- Use pg_regress as main test strategy.
