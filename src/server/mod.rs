use crate::interceptor::{Interceptor, InterceptorResult};
use crate::layer::Layer;
use crate::layer::LayerResult::{Cancel, Pass};
use crate::parser::Parsed;
use crate::routes::{ConnectionId, Dispatcher, Params, ServerRoutes, State};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::ops::Deref;

use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{ UnboundedSender};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

#[derive(Clone)]
pub struct ServerDispatcher {
    pub(crate) sender: tokio::sync::mpsc::UnboundedSender<String>,
    pub(crate) global_disp: tokio::sync::mpsc::UnboundedSender<GlobalDisp>,
}
impl ServerDispatcher {
    pub fn send(&self, msg: impl Into<String>) {
        let _ = self.sender.send(msg.into()).unwrap();
    }

    pub fn send_to(&self, msg: impl Into<String>, uuid: impl Into<String>) {
        let uuid = Uuid::from_str(&uuid.into()).unwrap();
        let gd = GlobalDisp {
            to: uuid,
            msg: msg.into(),
        };
        self.global_disp.send(gd).unwrap();
    }
}

pub(crate) struct GlobalDisp {
    pub(crate) msg: String,
    pub(crate) to: Uuid,
}

pub struct Server<S> {
    url: String,
    routes: Arc<Mutex<Vec<ServerRoutes<S>>>>,
    state: State<S>,
    layers: Vec<Layer<S>>,
    interceptor: Arc<Option<Interceptor>>,
    connections: Arc<Mutex<HashMap<Uuid, UnboundedSender<String>>>>,
}

impl<S: Send + Sync + 'static> Server<S> {
    pub fn new(url: impl Into<String>, state: S) -> Self {
        let url = url.into();
        Self {
            url,
            routes: Arc::new(Mutex::new(Vec::new())),
            state: State::new(state),
            layers: Vec::new(),
            interceptor: Arc::new(None),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn intercept(&mut self, interceptor: Interceptor) {
        self.interceptor = Arc::new(Some(interceptor));
    }

    pub fn layer(&mut self, layer: Layer<S>) {
        self.layers.push(layer);
    }

    pub async fn route<F, Fut>(&mut self, name: impl Into<String>, callback: F)
    where
        F: Fn(Params, ServerDispatcher, State<S>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let name = name.into();
        self.routes.lock().await.push(ServerRoutes {
            name,
            callback: Arc::new(move |params, dispatcher, state| {
                Box::pin(callback(params, dispatcher, state))
            }),
        });
    }
    pub async fn serve(&self) {
        //clones connections from self
        let connections: Arc<Mutex<HashMap<Uuid, UnboundedSender<String>>>> =
            self.connections.clone();

        //creates a global dispatcher - to send msg from one client to the other
        let (global_tx, mut global_rx) = tokio::sync::mpsc::unbounded_channel::<GlobalDisp>();

        let connections_clone = self.connections.clone();
        tokio::spawn(async move {
            while let Some(msg) = global_rx.recv().await {
                let locked = connections_clone.lock().await;
                if locked.contains_key(&msg.to) {
                    match locked.get(&msg.to).unwrap().send(msg.msg) {
                        Ok(_) => (),
                        Err(_) => (),
                    }
                }
            }
        });

        //clone everything from the self, that can't be used (can't move)
        println!("SERVING");
        let layers = &self.layers;
        let listener = TcpListener::bind(&self.url).await.unwrap();
        let routes = self.routes.clone();
        let state = self.state.clone();

        //in this while, there's all the client's connected
        while let Ok((stream, _)) = listener.accept().await {
            //clone stuff before moving into the tokio::spawn
            let routes = Arc::clone(&routes);
            let state = state.clone();
            let layers = layers.clone();
            let connections = Arc::clone(&connections);
            let tx_copy = global_tx.clone();
            let interceptor = self.interceptor.clone();

            //spawns a new task for every client
            tokio::spawn(async move {
                //tries to connect
                let ws = match accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(_) => return,
                };
                //split the stream
                let (mut write, mut read) = ws.split();

                //create an internal channel for communication between the crate and the user
                let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<String>();

                //create copy of the layers
                let layers_copy = layers.clone();
                let interceptor_copy = interceptor.clone();

                //create uuid
                let conn_id = ConnectionId(Uuid::new_v4());

                //saves the connection to be able to call it
                connections.lock().await.insert(conn_id.0, sender.clone());

                //tries to find the CONNECTED route to send the msg
                if let Some(route) = routes.lock().await.iter().find(|r| r.name == "CONNECTED") {
                    let params: Params =
                        Params::from([("uuid".to_string(), conn_id.0.to_string())]);
                    let callback = route.callback.clone();
                    let dispatcher = ServerDispatcher {
                        sender: sender.clone(),
                        global_disp: tx_copy.clone(),
                    };
                    let state = state.clone();

                    //awaits to all the layers to pass. if they fail, then the route stops executing
                    tokio::spawn(async move {
                        if run_layer(
                            "CONNECTED".to_string(),
                            layers.as_ref(),
                            dispatcher.clone(),
                            state.clone(),
                            params.clone(),
                        )
                        .await
                        {
                            println!("CONNECTED ALERT CALLED - RUN_LAYER PASSED");
                            callback(params, dispatcher, state).await;
                        }
                    });
                }

                //creates listeners for the internal channel and for the ws
                loop {
                    tokio::select! {


                            // outgoing
                            Some(msg) = receiver.recv() => {
                                let _ = write.send(Message::text(msg)).await;
                            }


                            // incoming
                            Some(Ok(msg)) = read.next() => {


                                let guard = interceptor_copy.clone();
                                let msg:String = match guard.deref() {
                                Some(interceptor) => {
                                    if let InterceptorResult::Pass(string) = (interceptor.callback)(msg.to_string(), Dispatcher{ sender: sender.clone()}).await {
                                        string
                                    }
                                    else {
                                        return;
                                    }
                                }
                                None => {msg.to_string()}
                            };

                                //tries to get the params and the command
                                let mut parsed = Parsed::parse(msg.to_string());
                                parsed.params.insert("uuid".to_string(), conn_id.0.to_string());
                                if let Some(route) =
                                    routes.lock().await.iter().find(|r| r.name == parsed.command)
                                {
                                    //runs the route
                                    let callback = route.callback.clone();
                                    let dispatcher = ServerDispatcher { sender: sender.clone(),
                    global_disp: tx_copy.clone()};
                                    let state = state.clone();
                                    let name = parsed.command.clone();
                                    let layers = layers_copy.clone();
                                    tokio::spawn(async move {
                                    if run_layer(name, layers.clone().as_ref(), dispatcher.clone(), state.clone(), parsed.params.clone() ).await {
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

                //when the code reaches here, the client disconnected...
                connections.lock().await.remove(&conn_id.0);
                if let Some(route) = routes
                    .lock()
                    .await
                    .iter()
                    .find(|r| r.name == "DISCONNECTED")
                {
                    let params: Params =
                        Params::from([("uuid".to_string(), conn_id.0.to_string())]);
                    let callback = route.callback.clone();
                    let dispatcher = ServerDispatcher {
                        sender: sender.clone(),
                        global_disp: tx_copy.clone(),
                    };
                    let state = state.clone();
                    let layers = layers_copy.clone();
                    tokio::spawn(async move {
                        if run_layer(
                            "DISCONNECTED".to_string(),
                            layers.as_ref(),
                            dispatcher.clone(),
                            state.clone(),
                            params.clone(),
                        )
                        .await
                        {
                            callback(params, dispatcher, state).await;
                        }
                    });
                }
            });
        }
    }
}

async fn run_layer<S: Send + Sync + 'static>(
    route: String,
    layers: &Vec<Layer<S>>,
    serverdisp: ServerDispatcher,
    state: State<S>,
    params: Params,
) -> bool {
    let mut params = params;
    println!("LAYERING CALLED");
    for layer in layers {
        if layer.blocked.contains(&route) {
            return false;
        }
        if !layer.allowed.contains(&route) && layer.allowed.len() > 0 {
            return false;
        }
        let parsed = params.clone();
        let callback = layer.callback.clone();
        let dispatcher = serverdisp.clone();
        let state = state.clone();
        let res = callback(parsed, dispatcher, state).await;
        match res {
            Pass(p) => {
                params = p;
            }
            Cancel => {
                return false;
            }
        }
    }
    true
}
