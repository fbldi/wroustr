
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
        let mut server = Server::new("127.0.0.1:3000".parse().unwrap());
        server.route("@NAME".to_string(), |params, disp| {
            disp.send("@THANKS ".to_string());
        });
        server.serve().await;
    }
}

