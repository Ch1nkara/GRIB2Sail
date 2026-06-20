## [0.7.2] - 2026-06-20

### 🐛 Bug Fixes

- Implement GribError from String

### 🚜 Refactor

- Get rid of external tool CDO
## [0.7.1] - 2026-04-15

### 🐛 Bug Fixes

- Increase iridium timeout to 120 seconds

### 🚜 Refactor

- Fetch_url_5_try to reduce duplicates
## [0.7.0] - 2026-03-27

### 🚀 Features

- Added ARPEGE model
- Added ECMWF model

### 🐛 Bug Fixes

- Typo in warning message
- Warning unsued variable e
- Concurrency not working

### 🚜 Refactor

- Keyring meteofrance instead of arome
- Noaa Step in url generation
- Implemented GribError::from
- Change lat lon separator , to :
- Change ReqwestData urls to urls_headers

## [0.6.1] - 2026-03-14

### 🐛 Bug Fixes

- Less gfs requests for same result
- Readme in crates.io

### 🚜 Refactor

- Retry up to 3 times if a request failed
## [0.6.0] - 2026-03-11

### 🚀 Features

- Added iridium module
- Added 2 gfs models 050 and 100

### 🐛 Bug Fixes

- Improved tests with sleeps
- Progress bar showing too early
- Handle ignored Results
- Cover lacking elements in gfs tests
- Gfs 050 and 100 lacking 1h step

### 🚜 Refactor

- Replace a direct call to imported func
- Fetch_data retry download 3 times
- Default to gfs instead of arome

### 📚 Documentation

- Updated docs and unit tests
## [0.5.2] - 2026-03-01

### 🐛 Bug Fixes

- Use HEAD instead of GET to find gfs latest
- Arome sleeping 1 min after finishing dl
- Unecessary borrows in updater
- Cli logger messing progress bar
## [0.5.1] - 2026-02-25

### 🐛 Bug Fixes

- Self-update compression and path
## [0.5.0] - 2026-02-24

### 🚜 Refactor

- Migrate the code to Rust
## [0.4.0] - 2026-02-05

- Fallback 1 run if the last one is incomplete
- Remove unecessary arome error handling
- Add full module tests
- Allow user to choose output directory
- Arome error bypass when missing layer
- Optimize test duration

## [0.3.1] - 2026-02-05

- Fix arome outfile naming pattern
- Error in GFS when when step=1h and days>5
- Unavailable layers in GFS latest run

## [0.3.0] - 2026-02-04

- Add the others arome models
- Indentation issues and default value
- Improve function write_file reusability
- Add --days option
- Add GFS model resolution 0025

## [0.2.1] - 2026-02-01

- Add a logo to the project
- Create a github release via Github actions

## [0.2.0] - 2026-01-30

- Fix typo in README
- Typo in .gitignore
- Improve logged information
- Improve AROME download speed
- Separate arome logic from common logic
- Gracefully handle arome requests limitations

## [0.1.0] - 2026-01-27

- Arome antille basic download
