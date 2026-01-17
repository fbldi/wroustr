//! # wsrouter
//!
//! Fast and easy WebSocket message routing for servers and clients.
//!
//! ## Overview
//!
//! `wsrouter` provides a simple message-based routing layer for WebSocket
//! servers and clients using a lightweight parser.
//!
//! ## Features
//!
//! - `server` – Enables the WebSocket server
//! - `client` – Enables the client-side connector
//!
//!
//! ## Routing
//!
//! - `route(name:text, callback:|params, dispatcher|{})` – Registers a route handler.
//! - `dispatcher`   - sends a response. on the server only responses to one client.
//! - `CONNECTED`    - Have to be in a route() function. Called when a new connection is established.
//! - `DISCONNECTED` - Have to be in a route() function. Called when a connection is closed.
//! - `@`            - Custom routes. Called when a client sends a message with the specified command.
//!
//! ## Authentication
//!
//! The uuid is always provided as a parameter in the callback functions.
//! This crate does not manage authentication or user state beyond providing this identifier.
//!
//! ## Example
//!
//! ### Server
//!
//! ```no_run
//! # #[tokio::main]
//! # async fn main() {
//! use wroustr::server::Server;
//!
//! let mut server = Server::new("127.0.0.1:3000");
//! server.route("@PING", |_, dispatcher,_| async move {
//!     dispatcher.send("@PONG");
//! });
//!
//! server.route("CONNECTED", |params, dispatcher,_| async move  {
//!     println!("New connection: {}", params.get("uuid").unwrap());
//! });
//!
//! server.route("DISCONNECTED", |params, dispatcher,_| async move {
//!     println!("Connection closed: {}", params.get("uuid").unwrap());
//! });
//!
//!
//! server.serve().await;
//! # }
//! ```
//!
//! ### Client
//!
//! ```no_run
//! # #[tokio::main]
//! # async fn main() {
//! use wroustr::client::Connector;
//!
//! let connector = Connector::new("ws://localhost:3000");
//! connector.route("@PING", |_, dispatcher,_| async move{
//!     dispatcher.send("@PONG");
//! });
//!
//! let dispatcher = connector.connect().await;
//! dispatcher.send("@PING");
//! dispatcher.keep_alive().await;
//! # }
//! ```

//use crate::routes::{Params, State};
#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;
pub mod routes;
#[cfg(feature = "interception")]
pub mod interceptor;
#[cfg(feature = "layers")]
pub mod layer;
mod parser;



// #[cfg(all(test, feature = "server"))]
// mod tests {
//
//     use crate::layer::{Layer, LayerResult};
//     use crate::layer::LayerResult::Cancel;
//     use crate::server::{Server, ServerDispatcher};
//     use super::*;
//
//     async fn init(params: Params, dispatcher: ServerDispatcher, state: State<&str>) {
//         println!("INIT: {:?}", params);
//         println!("State: {:?}", state);
//         dispatcher.send("@INIT-DONE #did-you-login? YEEEES ");
//     }
//
//     async fn auth(params: Params, dispatcher: ServerDispatcher, _state: State<&str>) -> LayerResult {
//         println!("AUTH: {:?}", params);
//         dispatcher.send("@AUTH.FAILED #error not-gonna-tell-you");
//         Cancel
//     }
//
//
//     #[tokio::test]
//     async fn test_serving() {
//         let appstate = "anything can be appstate!";
//
//         let auth_layer = Layer::new("AUTH",auth)
//             .block(vec!["CONNECTED"]);
//
//
//         let mut server = Server::new("localhost:3000", appstate);
//         server.layer(auth_layer);
//         server.route("@INIT", init).await;
//         server.route("CONNECTED", |_map, dispatcher, _arc| async move {
//             println!("CONNECTED!");
//             dispatcher.send("@DEINIT");
//         }).await;
//
//         server.serve().await;
//     }
//
//
// }

