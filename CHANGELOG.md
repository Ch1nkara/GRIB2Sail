## [0.5.0-alpha.7] - 2026-02-22

### 🚀 Features

- Added dummy meteofrance module and async
- Imporved downloader but not by much
- Add GribError and added stuff meteofrance
- Keyring in cli and more
- Added meteofrance config and token
- Working arome token getter and latestrun
- Working simple arome dl
- Added all arome models
- Handle more than 100 arome layers
- Added model gfs and lint via rustfmt

### 💼 Other

- Rename utils submodule core

### 🚜 Refactor

- Improve module inbetween calls
- Simplify internal modules organisation
- Polishing and linting with clippy
## [0.5.0-alpha.6] - 2026-02-09

### 🚀 Features

- Added progress bar, Results and more

### 🐛 Bug Fixes

- Clarify lib import in main

### 🚜 Refactor

- Added cli module almost functionnal
## [0.5.0-alpha.4] - 2026-02-08

### 💼 Other

- Workspace missing on root directory
- V0.5.0-alpha.2
- V0.5.0-alpha.3
- V0.5.0-alpha.4
## [0.5.0-alpha.1] - 2026-02-08

### 💼 Other

- Fallback 1 run if the last one is incomplete
- Remove unecessary arome error handling
- Allow user to choose output directory
- Arome error bypass when missing layer
- First commit for migration to rust
- Reorganizing as unique crate
## [0.3.1] - 2026-02-05

### 💼 Other

- Fix arome outfile naming pattern
- Error in GFS when when step=1h and days>5
- Unavailable layers in GFS latest run
## [0.3.0] - 2026-02-04

### 💼 Other

- Add the others arome models
- Indentation issues and default value
- Improve function write_file reusability
- Add --days option
- Add GFS model resolution 0025
## [0.2.0] - 2026-01-30

### 💼 Other

- Improve logged information
- Improve AROME download speed
- Separate arome logic from common logic
- Gracefully handle arome requests limitations
## [0.1.0] - 2026-01-27

### 💼 Other

- Arome antille basic download
