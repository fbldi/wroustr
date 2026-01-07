use std::slice::Iter;
use crate::parser::Parsed;
use crate::routes::{ConnectionId, Dispatcher, Params, Route, State};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;
use crate::layer::Layer;

pub struct Server<S> {
    url: String,
    routes: Vec<Route<S>>,
    state: State<S>,
    layers: Vec<Layer<S>>,
}

impl<S: Send + Sync + 'static> Server<S> {
    pub fn new(url: impl Into<String>, state: S) -> Self {
        let url = url.into();
        Self {
            url,
            routes: Vec::new(),
            state: State::new(state),
            layers: Vec::new(),
        }
    }

    pub fn layer(&mut self, layer: Layer<S>) {
        self.layers.push(layer);
    }




    pub fn route<F, Fut>(&mut self, name: impl Into<String>, callback: F)
    where
        F: Fn(Params, Dispatcher, State<S>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let name = name.into();
        self.routes.push(Route {
            name,
            callback: Arc::new(move |params, dispatcher, state| {
                Box::pin(callback(params, dispatcher, state))
            }),
        });
    }
    pub async fn serve(self) {
        let layers = self.layers;
        let listener = TcpListener::bind(&self.url).await.unwrap();
        let routes = Arc::new(self.routes);
        let state = self.state.clone(); //
        while let Ok((stream, _)) = listener.accept().await {
            let routes = Arc::clone(&routes);
            let state = state.clone();
            let layers =layers.clone();
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
                let layers_copy = layers.clone();
                //create uuid
                let conn_id = ConnectionId(Uuid::new_v4());
                if let Some(route) = routes.iter().find(|r| r.name == "CONNECTED") {
                    let params: Params = Params::from([("uuid".to_string(), conn_id.0.to_string())]);
                    let callback = route.callback.clone();
                    let dispatcher = Dispatcher { sender: sender.clone() };
                    let state = state.clone();
                    let sender = sender.clone();

                    tokio::spawn(async move {
                        if run_layer("CONNECTED".to_string(), layers.as_ref(), sender.clone(), state.clone(), params.clone()).await {
                            callback(params, dispatcher, state).await;
                        }
                    });
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
                                    let callback = route.callback.clone();
                                    let dispatcher = Dispatcher { sender: sender.clone()};
                                    let state = state.clone();
                                    let name = parsed.command.clone();
                                    let layers = layers_copy.clone();
                                    let sender = sender.clone();
                                    tokio::spawn(async move {
                                    if run_layer(name, layers.clone().as_ref(), sender.clone(), state.clone(), parsed.params.clone() ).await {
                                        callback(parsed.params, dispatcher, state).await;
                                    }
                                    });
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
                    let callback = route.callback.clone();
                    let dispatcher = Dispatcher { sender: sender.clone() };
                    let state = state.clone();
                    let layers = layers_copy.clone();
                    tokio::spawn(async move {
                        if run_layer("DISCONNECTED".to_string(), layers.as_ref(), sender.clone(), state.clone(), params.clone()).await {
                            callback(params, dispatcher, state).await;
                        }
                    });
                }
            });
        }
    }
}


async fn run_layer<S: Send + Sync + 'static>(route: String, layers: &Vec<Layer<S>>, sender: UnboundedSender<String>, state: State<S>, params: Params) -> bool {
    for layer in layers {
        if layer.blocked.contains(&route) {
            return false;
        }
        if !layer.allowed.contains(&route) && layer.allowed.len() > 0 {
            return false;
        }
        let parsed = params.clone();
        let callback = layer.callback.clone();
        let dispatcher = Dispatcher { sender: sender.clone() };
        let state = state.clone();
        let res = callback(parsed, dispatcher, state).await;
        if !res {
            return false;
        }
    }
    true
}