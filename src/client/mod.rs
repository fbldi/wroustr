use std::ops::Deref;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use crate::interceptor::{Interceptor, InterceptorResult};
use crate::parser::Parsed;
use crate::routes::{Dispatcher, Params, Route, State};

pub struct Connector<S> {
    url: String,
    routes: Vec<Route<S>>,
    interceptor: Arc<Option<Interceptor>>,
    state: State<S>
}







impl<S: Send + Sync + 'static> Connector<S> {
    //create new connector with an url
    pub fn new(url: impl Into<String>, state: S) -> Self {
        let url = url.into();
        Self {
            url,
            routes: Vec::new(),
            state: State::new(state),
            interceptor: Arc::new(None)
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

    pub fn intercept(&mut self, interceptor: Interceptor) {
        self.interceptor = Arc::new(Some(interceptor));
    }

    //connect to the server by consuming this connector and returning a dispather
    pub async fn connect(self) -> Dispatcher {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<String>();
        let routes = self.routes;
        let sender_clone = sender.clone();
        let url = Arc::new(self.url);
        
        
        //move everything in one spawn to manage reconnection
        let _life_cycle = tokio::spawn(async move {
            loop {
                
                
                //connect to the server
                let (ws_stream, _) = match connect_async(url.to_string()).await  {
                    Ok(stream) => stream,
                    Err(e) => {
                        eprintln!("connect error: {}", e);
                        continue;
                    }
                };
                
                
                //split ws into read-write
                let (mut write, mut read) = ws_stream.split();

                //clones the interceptor
                let interceptor = self.interceptor.clone();
                
                //tries to send CONNECTED alert
                if let Some(found_route) = routes.iter().find(|route| route.name == "CONNECTED") {
                    let params =Params::new();
                    let state = self.state.clone();
                    let callback = found_route.callback.clone();
                    let dispatcher = Dispatcher { sender: sender_clone.clone() };
                    tokio::spawn(async move {
                        callback(params, dispatcher, state).await;
                    });
                }
                
                
                //runs the actual connection inside
                //the connection_result var can output why the connection closed
                let connection_result = loop {
                    tokio::select! {
                        
                        
                        //DISPATCHING
                        //waits to get an alert from the Dispatcher
                        Some(msg) = receiver.recv() => {
                            if let Err(e) = write.send(Message::text(msg)).await {
                                eprintln!("send error: {}", e);
                                break;
                            }
                        },
                        
                        
                        //RECEIVING
                        //waits to get an alert from the WS
                        //tries to parse it and send out an alert
                        Some(Ok(msg)) = read.next() => {
                            let guard = interceptor.clone();
                            let msg:String = match guard.deref() {
                                Some(interc) => {
                                    if let InterceptorResult::Pass(string) = (interc.callback)(msg.to_string(), Dispatcher{sender: sender_clone.clone()}).await {
                                        string
                                    }
                                    else {
                                        return
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
                                let dispatcher = Dispatcher { sender: sender_clone.clone() };
                                tokio::spawn(async move {
                                    callback(params, dispatcher, state).await;
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
                if let Some(found_route) = routes.iter().find(|route| route.name == "DISCONNECT") {
                    let params =Params::new();
                    let state = self.state.clone();
                    let callback = found_route.callback.clone();
                    let dispatcher = Dispatcher { sender: sender_clone.clone() };

                    tokio::spawn(async move {
                        callback(params, dispatcher, state).await;
                    });
                }
                
                
                //waits 2s before trying to reconnect
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        });

        Dispatcher { sender }
    }
}