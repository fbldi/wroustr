pub mod client;
mod server;
mod routes;
mod parser;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use crate::client::Connector;
    use crate::server::Server;
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    // #[tokio::test]
    // async fn it_works_too() {
    //     println!("Hello, world!");
    //     let mut connector = Connector::new("ws://localhost:3000/ws".parse().unwrap());
    //     connector.route("@NAME".to_string(), |params, disp| {
    //         println!("{:?}", params);
    //         disp.send("@THANKS ".to_string());
    //     });
    //     connector.connect().await;
    // }

    #[tokio::test]
    async fn test_serving() {
        let mut server = Server::new("127.0.0.1:3000".parse().unwrap());
        server.route("@NAME".to_string(), |params, disp| {
            disp.send("@THANKS ".to_string());
        });
        server.serve().await;
    }
}

