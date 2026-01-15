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


## [0.5.1] - 2026.01.12
## Added
- Intercepting (instable)
  - Interceptor Struct
  - InterceptorResult Enum (Pass(String) or Cancel)
  - Intercept func for client and server

## [0.5.2] - 2026.01.15
### Added 
- InterceptorType
  - INCOMING
  - OUTGOING
### Changed
- Interception is now runs when a message arrives and can run when a message is sent.