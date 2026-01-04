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
//! - `route`  – Provides a simple routing API
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
//! let mut server = Server::new("127.0.0.1:3000".to_string());
//! server.route("@PING".to_string(), |_, dispatcher| {
//!     dispatcher.send("@PONG".to_string());
//! });
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
//! let connector = Connector::new("ws://localhost:3000".to_string());
//! connector.route("@PING".to_string(), |_, dispatcher| {
//!     dispatcher.send("@PONG".to_string());
//! });
//!
//! let dispatcher = connector.connect().await;
//! dispatcher.send("@PING".to_string());
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

