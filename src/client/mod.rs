use std::ops::Deref;
#[cfg(feature = "interception")]
use crate::interceptor::{
    Interceptor,
    InterceptorResult,
    InterceptorType,
    ServerInterceptor,
};

#[cfg(feature = "layers")]
use crate::layer::ClientLayer;
#[cfg(feature = "layers")]
use crate::layer::LayerResult::{Cancel, Pass};
use crate::parser::Parsed;
use crate::routes::{Dispatcher, Params, Route, State};
use futures_util::{SinkExt, StreamExt};

use std::sync::Arc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

pub struct Connector<S> {
    url: String,
    routes: Vec<Route<S>>,
    #[cfg(feature = "interception")]
    incoming_ir: Arc<Option<Interceptor<S>>>,
    #[cfg(feature = "interception")]
    outgoing_ir: Arc<Option<Interceptor<S>>>,
    #[cfg(feature = "layers")]
    layers: Vec<ClientLayer<S>>,
    state: State<S>,
}

impl<S: Send + Sync + 'static> Connector<S> {
    //create a new connector with a url
    pub fn new(url: impl Into<String>, state: S) -> Self {
        let url = url.into();
        Self {
            url,
            routes: Vec::new(),
            state: State::new(state),
            #[cfg(feature = "layers")]
            layers: Vec::new(),
            #[cfg(feature = "interception")]
            incoming_ir: Arc::new(None),
            #[cfg(feature = "interception")]
            outgoing_ir: Arc::new(None),
        }
    }

    //add new incomeing routes

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

    #[cfg(feature = "layers")]
    pub fn layer(&mut self, layer: ClientLayer<S>) {
        self.layers.push(layer);
    }

    #[cfg(feature = "interception")]
    pub fn intercept(&mut self, interceptor: Interceptor<S>) {
        if interceptor.r#type == InterceptorType::INCOMING {
            self.incoming_ir = Arc::new(Some(interceptor));
        } else {
            self.outgoing_ir = Arc::new(Some(interceptor));
        }
    }

    //connect to the server by consuming this connector and returning a dispather
    pub async fn connect(self) -> Dispatcher {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<String>();
        let routes = self.routes;
        let sender_clone = sender.clone();
        let url = Arc::new(self.url);
        #[cfg(feature = "layers")]
        let layers = self.layers.clone();

        //move everything in one spawn to manage reconnection
        let _life_cycle = tokio::spawn(async move {
            loop {
                //connect to the server
                let (ws_stream, _) = match connect_async(url.to_string()).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        eprintln!("connect error: {}", e);
                        continue;
                    }
                };

                //split ws into read-write
                let (mut write, mut read) = ws_stream.split();

                //clones the interceptor
                #[cfg(feature = "interception")]
                let incoming_ir = self.incoming_ir.clone();
                #[cfg(feature = "interception")]
                let outgoing_ir = self.outgoing_ir.clone();

                //tries to send CONNECTED alert
                if let Some(found_route) = routes.iter().find(|route| route.name == "CONNECTED") {
                    let params = Params::new();
                    let state = self.state.clone();
                    let callback = found_route.callback.clone();
                    let dispatcher = Dispatcher {
                        sender: sender_clone.clone(),
                    };
                    #[cfg(feature = "layers")]
                    let layers = layers.clone();
                    tokio::spawn(async move {
                        #[cfg(feature = "layers")]
                        if run_layer(
                            "CONNECTED".to_string(),
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

                //runs the actual connection inside
                //the connection_result var can output why the connection closed
                let connection_result = loop {
                    tokio::select! {


                        //DISPATCHING
                        //waits to get an alert from the Dispatcher
                        Some(msg) = receiver.recv() => {
                            #[cfg(feature = "interception")]
                            let guard = outgoing_ir.clone();
                            #[cfg(feature = "interception")]
                                let msg:String = match guard.deref() {
                                Some(interceptor) => {
                                    if let InterceptorResult::Pass(string) = (interceptor.callback)(msg.to_string(), self.state.clone()).await {
                                        string
                                    }
                                    else {
                                        break;
                                    }
                                }
                                None => {msg.to_string()}
                            };
                                let _ = write.send(Message::text(msg)).await;
                            }


                        //RECEIVING
                        //waits to get an alert from the WS
                        //tries to parse it and send out an alert
                        Some(Ok(msg)) = read.next() => {

                            let msg = match msg {
                                Message::Text(t) => t,
                                _ => break
                            };

                            #[cfg(feature = "interception")]
                            let guard = incoming_ir.clone();
                            #[cfg(feature = "interception")]
                            let msg:String = match guard.deref() {
                                Some(interc) => {
                                    if let InterceptorResult::Pass(string) = (interc.callback)(msg.to_string(), self.state.clone()).await {
                                        string
                                    }
                                    else {
                                        break
                                    }
                                },
                                None => {
                                    msg.to_string()
                                }
                            };



                            //tries to read out the message and parse it to be an alert msg
                            //if it fails, then it returns whitespace
                            let parsed = Parsed::parse(msg.to_string());
                            let params = parsed.params;
                            let command = parsed.command;


                            //copy-s the appstate
                            //IMPORTANT! the appstate is not mutable! ONLY the fields of the state can be mut
                            let state = self.state.clone();



                            //tries to find a route with the command name
                            if let Some(found_route) = routes.iter().find(|route| route.name == command.to_string()) {

                                //creates the future and runs it down
                                let callback = found_route.callback.clone();
                                let name = found_route.name.clone();
                                let dispatcher = Dispatcher { sender: sender_clone.clone() };
                                #[cfg(feature = "layers")]
                                let layers = layers.clone();
                                tokio::spawn(async move {
                                     if run_layer(name,
                                        #[cfg(feature = "layers")]
                                        layers.clone().as_ref(), dispatcher.clone(), state.clone(), params.clone() ).await {
                                    callback(params, dispatcher, state).await;
                                        }
                                });
                                }
                            else {
                                println!("No route found for {}", command);
                                println!("Current routes: {:?}", routes.iter().map(|route| route.name.clone()).collect::<Vec<String>>());
                                }
                            }


                        //on error or if connection to either direction lost, breaks the loop
                        else => break
                    }
                };
                eprintln!("Connection error: {:?}", connection_result);

                //tries to find and alert the DISCONNECTED route
                if let Some(found_route) = routes.iter().find(|route| route.name == "DISCONNECTED") {
                    #[cfg(feature = "layers")]
                    let layers = layers.clone();
                    let params = Params::new();
                    let state = self.state.clone();
                    let callback = found_route.callback.clone();
                    let dispatcher = Dispatcher {
                        sender: sender_clone.clone(),
                    };

                    tokio::spawn(async move {
                        if run_layer(
                            "DISCONNECTED".to_string(),
                            #[cfg(feature = "layers")]
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

                //waits 2s before trying to reconnect
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        });

        Dispatcher { sender }
    }
}

async fn run_layer<S: Send + Sync + 'static>(
    route: String,
    #[cfg(feature = "layers")]
    layers: &Vec<ClientLayer<S>>,
    clientdisp: Dispatcher,
    state: State<S>,
    params: Params,
) -> bool {
    let mut params = params;
    println!("LAYERING CALLED");
    #[cfg(feature = "layers")]
    for layer in layers {
        if layer.blocked.contains(&route) {
            return false;
        }
        if !layer.allowed.contains(&route) && layer.allowed.len() > 0 {
            return false;
        }
        let parsed = params.clone();
        let callback = layer.callback.clone();
        let dispatcher = clientdisp.clone();
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
