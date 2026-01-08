# Changelog
All notable changes to this crate will be documented in this file from v0.5.0

## [0.5.0] - 2026.01.08
### Added
- ServerDispatcher struct for the server
- global command dispatching bus for ServerDispatcher
- connection tracking on the server

### Changed
- route() function is now asynchronous
- layers must return a LayerResult with parameters if passed
- removed keep_alive from ServerDispatcher

### Fixed
- Reconnection errors for the client.
- Stack overflow bug on layers caused by recursive functions