
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use crate::parser::Parsed;
use crate::routes::{Dispatcher, Params, Route};

pub struct Connector {
    url: String,
    routes: Vec<Route>
}







impl Connector {
    //create new connector with an url
    pub fn new(url: impl Into<String>) -> Self {
        let url = url.into();
        Self {
            url,
            routes: Vec::new()
        }
    }

    //add new incomeing routes
    pub fn route<F>(&mut self, name: impl Into<String>, callback: F)
    where
        F: Fn(&Params, &Dispatcher) + Send + Sync + 'static,
    {
        let name = name.into();
        self.routes.push(Route {
            name,
            callback: Box::new(callback),
        });
    }

    //connect to the server by consuming this connector and returning a dispather
    pub async fn connect(self) -> Dispatcher {

        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<String>();

        let (ws_stream, _) = connect_async(self.url).await.unwrap();
        let (mut write, mut read) = ws_stream.split();



        // DISPATCHING MESSAGES
        let _dispatching = tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                println!("Received: {}", msg);
                // itt jön majd a parser + router dispatch
                write.send(Message::text(msg)).await.unwrap();
            }
        });

        let sender_clone = sender.clone();
        let routes = self.routes;

        // HANDLING INCOMING MESSAGES
        let _receiving = tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                let parsed = Parsed::parse(msg.unwrap().to_string());
                let params = parsed.params;
                let command = parsed.command;

                if let Some(found_route) = routes.iter().find(|route| route.name == command.to_string()) {
                    (found_route.callback)(&params, &Dispatcher { sender: sender_clone.clone() });
                }
                else {
                    println!("No route found for {}", command);
                    println!("Current routes: {:?}", routes.iter().map(|route| route.name.clone()).collect::<Vec<String>>());
                }
            }
        });

        Dispatcher { sender }



    }
}