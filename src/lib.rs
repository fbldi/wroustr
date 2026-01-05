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
//! use wsrouter::server::Server;
//!
//! let mut server = Server::new("127.0.0.1:3000");
//! server.route("@PING", |_, dispatcher| {
//!     dispatcher.send("@PONG");
//! });
//!
//! server.route("CONNECTED", |params, dispatcher| {
//!     println!("New connection: {}", params.get("uuid").unwrap());
//! });
//!
//! server.route("DISCONNECTED", |params, dispatcher| {
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
//! use wsrouter::client::Connector;
//!
//! let connector = Connector::new("ws://localhost:3000");
//! connector.route("@PING", |_, dispatcher| {
//!     dispatcher.send("@PONG");
//! });
//!
//! let dispatcher = connector.connect().await;
//! dispatcher.send("@PING");
//! dispatcher.keep_alive().await;
//! # }
//! ```



#[cfg(feature = "client")]
pub mod client;


#[cfg(feature = "server")]
pub mod server;
mod routes;
mod parser;


#[cfg(all(test, feature = "server"))]
mod tests {
    use crate::client::Connector;
    use crate::server::Server;
    use super::*;
    #[tokio::test]
    async fn test_serving() {
        let mut server = Server::new("127.0.0.1:3000");
        server.route("@NAME", |params, disp| {
            disp.send("@THANKS ");
        });
        server.serve().await;
    }
}

