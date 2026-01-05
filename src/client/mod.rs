use std::sync::Arc;
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
        let routes = self.routes;
        let sender_clone = sender.clone();
        let url = Arc::new(self.url);
        //let (mut write, mut read) = ws_stream.split();

        //move everything in one spawn to manage reconnection
        let _life_cycle = tokio::spawn(async move {
            loop {
                let (ws_stream, _) = match connect_async(url.to_string()).await  {
                    Ok(stream) => stream,
                    Err(e) => {
                        eprintln!("connect error: {}", e);
                        continue;
                    }
                };
                let (mut write, mut read) = ws_stream.split();

                if let Some(found_route) = routes.iter().find(|route| route.name == "CONNECTED") {
                    let params =&Params::new();
                    (found_route.callback)(&params, &Dispatcher { sender: sender_clone.clone() });
                }
                let connection_result = loop {
                    tokio::select! {
                        Some(msg) = receiver.recv() => {
                            if let Err(e) = write.send(Message::text(msg)).await {
                                eprintln!("send error: {}", e);
                                break;
                            }
                        },
                        Some(msg) = read.next() => {
                            let msg = match msg {
                                Ok(msg) => msg,
                                Err(e) => {
                                    eprintln!("couldn't extract message: {}", e);
                                    break
                                }
                            };
                            let parsed = Parsed::parse(msg.to_string());
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
                        else => break
                    }
                };
                eprintln!("Connection error: {:?}", connection_result);
                if let Some(found_route) = routes.iter().find(|route| route.name == "DISCONNECT") {
                    let params =&Params::new();
                    (found_route.callback)(&params, &Dispatcher { sender: sender_clone.clone() });
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        });

        Dispatcher { sender }
    }
}