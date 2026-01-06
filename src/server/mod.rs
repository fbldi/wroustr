use crate::parser::Parsed;
use crate::routes::{ConnectionId, Dispatcher, Params, Route, State};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

pub struct Server<S> {
    url: String,
    routes: Vec<Route<S>>,
    state: State<S>
}

impl<S: Send + Sync + 'static> Server<S> {
    pub fn new(url: impl Into<String>, state: S) -> Self {
        let url = url.into();
        Self {
            url,
            routes: Vec::new(),
            state: State::new(state)
        }
    }

    pub fn route<F, Fut>(&mut self, name: impl Into<String>, callback: F)
    where
        F: Fn(Params, Dispatcher, State<S>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let name = name.into();
        self.routes.push(Route {
            name,
            callback: Box::new(move |params, dispatcher, state| {
                Box::pin(callback(params, dispatcher, state))
            }),
        });
    }
    pub async fn serve(self) {
        let listener = TcpListener::bind(&self.url).await.unwrap();
        let routes = Arc::new(self.routes);
        let state = self.state.clone(); //
        while let Ok((stream, _)) = listener.accept().await {
            let routes = Arc::clone(&routes);
            let state = state.clone();
            tokio::spawn(async move {
                let ws = match accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(_) => return,
                };
                let (mut write, mut read) = ws.split();

                let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<String>();
                let dispatcher = Dispatcher {
                    sender: sender.clone(),
                };

                //create uuid
                let conn_id = ConnectionId(Uuid::new_v4());
                if let Some(route) = routes.iter().find(|r| r.name == "CONNECTED") {
                    let params: Params = Params::from([("uuid".to_string(), conn_id.0.to_string())]);
                    let fut = (route.callback)(params, Dispatcher { sender: sender.clone() }, state.clone());
                    tokio::spawn(async move { fut.await });
                }

                loop {
                    tokio::select! {
                            // outgoing
                            Some(msg) = receiver.recv() => {
                                let _ = write.send(Message::text(msg)).await;
                            }

                            // incoming
                            Some(Ok(msg)) = read.next() => {
                                let mut parsed = Parsed::parse(msg.to_string());
                                parsed.params.insert("uuid".to_string(), conn_id.0.to_string());
                                if let Some(route) =
                                    routes.iter().find(|r| r.name == parsed.command)
                                {
                                    let fut = (route.callback)(parsed.params, Dispatcher { sender: sender.clone() }, state.clone());
                                    tokio::spawn(async move { fut.await });
                                }
                            }

                            else => {



                                break
                            },
                        }
                }

                if let Some(route) = routes.iter().find(|r| r.name == "DISCONNECTED")
                {
                    let params: Params = Params::from([("uuid".to_string(), conn_id.0.to_string())]);
                    let fut = (route.callback)(params, Dispatcher { sender: sender.clone() }, state.clone());
                    tokio::spawn(async move { fut.await });
                }
            });
        }
    }
}
