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

use crate::routes::{Dispatcher, Params, State};
use crate::server::Server;
#[cfg(feature = "client")]
pub mod client;


#[cfg(feature = "server")]
pub mod server;
mod routes;
mod parser;


#[cfg(all(test, feature = "server"))]
mod tests {
    
    async fn init(params: Params, dispatcher: Dispatcher, state: State<&str>) {
        println!("INIT: {:?}", params);
        println!("State: {:?}", state);
        dispatcher.send("@INIT-DONE #did-you-login? YEEEES ");
    }
    use crate::client::Connector;
    use crate::server::Server;
    use super::*;
    #[tokio::test]
    async fn test_serving() {
        let mut server = Server::new("127.0.0.1:3000", "asdasd");
        server.route("@NAME", |params, disp, state|async move {
            disp.send("@THANKS ");
        });

        server.route("CONNECTED", |params, disp, state|async move {
            println!("New connection: {}", params.get("uuid").unwrap());
            println!("State: {:?}", state);
            disp.send("@CONNECTION-DONE #did-you-login? YEEEES ");
        });

        server.route("@INIT", init);

        server.serve().await;
    }
}

