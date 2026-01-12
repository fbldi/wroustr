## wroustr

**wroustr** is a simple routing layer on top of WebSocket connections,
creating routes based on message content.

## Features

- async WebSocket routing for client and server
- Named routes (e.g. `CONNECTED`, `DISCONNECTED`)
- Custom routes
- String parameters
- client tracking
- Tokio-based, non-blocking
- auto-reconnect

---

## Example



```rust
use wroustr::client::Connector;

#[tokio::main]
async fn main() {
let state = AppState { /* ... */ };

    let connector = Connector::new("ws://localhost:9000", state)
        .route("CONNECTED", |params, dispatcher, state| async move {
            println!("Client connected: {:?}", params);
        })
        .route("@PING", |params, dispatcher, state| async move {
            dispatcher.send("@PONG").await;
        });

    connector.connect("127.0.0.1:9000").await;
}
```
here the client connects to the server. 

the `CONNECTED` route is triggered automatically, and when the server
sends a `@PING` message, the client responds with `@PONG`.

```rust
    use wroustr::server::Server;    

    #[tokio::main]
    async fn test_serving() {
        let state = AppState { /* ... */ };
        let mut server = Server::new("127.0.0.1:3000", state);
        server.route("@LOGIN", |params, disp, state|async move {
            disp.send("@LOGIN-DONE #success true");
        }).await;
    
        server.route("CONNECTED", |params, disp, state|async move {
            println!("New connection: {}", params.get("uuid").unwrap());
            println!("State: {:?}", state);
            disp.send("@CONNECTION-ESTABLISHED");
        }).await;
    
        server.route("@INIT", init).await;
    
        server.serve().await;
    }


    async fn init(params: Params, dispatcher: Dispatcher, state: State<Appstate>) {
        println!("INIT: {:?}", params);
        println!("State: {:?}", state);
        dispatcher.send("@INIT-DONE #success true");
    }
```
here the server has 1 named route: `CONNECTED`, and 2 custom routes.
custom routes should always start with `@`.

Please note that the route() function changed for the server and is now async.


## Parameters
all parameters will be parsed as strings. the keys should always begin with `#`
and the value must be separated with a space.

on the server, there's always an uuid parameter to keep track of the client.

## Dispatcher
the Dispatcher struct is the client site websocket sender.
you can use the send() function to send a message to the server.

## ServerDispatcher
this struct is on the server.
you can use the send() function the same way, as on the client,
but now you have another function called send_to(msg: impl Into<String>, uuid: impl Into<String>) that will 
send the message to the assigned client. (you have to manage the uuids given by the server as the uuid parameter)

## Appstate
the state is passed to all routes, but by default it's immutable.
to create mutable states, use `Mutex`, `Atomic*`, `DashMap`, etc. as fields.


## Layers
You can use layers by adding the `layers` feature in Cargo.toml: 

```
[dependencies]
wroustr = {version = "0.3.4", features = ["layers"]}
```
Layers allow you to run middleware-like logic before routes execute.
Layers will always run when a route is called. You can also add `allowed` routes, and `blocked` routes.
Layers are only available on the server.
## Example
```rust
use wroustr::layers::Layer;
    use wroustr::server::Server;
    #[tokio::main]
    async fn main() {
        let appstate = "APPSTATE CAN BE ANYTHING";

        let layer = Layer::new("AUTH", |params, dispatcher, state: State<&str>|async move {
            //Some auth function
            //get token from the params
            let token = params.get("token").unwrap();
            //get user-id and run sql if needed to

            //layers MUST return a bool -> true = can proceed, false = layer failed, and now it's returning,
            //but you can still use the dispatcher from the layers if you need to return an answer to the client
            dispatcher.send("@AUTH-FAILED");
            true
        })
            //this means that the auth layer won't when a client connects or disconnects
            .block(vec!["CONNECTED, DISCONNECTED"])
            //allow() will enable a layer to run on specific routes.
            //if there are no allowed routes, then all routes are allowed
            .allow(vec!["@REQUEST-DATA"]);



        let mut server = Server::new("127.0.0.1:3000", appstate);
        //layer in the auth layer
        server.layer(layer);
        server.route("@REQUEST-DATA", |params, disp, state|async move {
            //do not have to check auth, because the AUTH layer takes care of that
            disp.send("@SEND-DATA #auth passed #data some-data ");
        });

        server.serve().await;
    }
```
NOTE: the layering system had been updated and is now considered stable.

## Intercepting

You can process the incoming message as you like with the Interceptors
Create one, then add it to the client or server with the intercept function
If there's an interceptor, the raw message will be sent to your callback function, that must return an InterceptorResult with the 
new/prossesed message. Useful for encryption.

NOTE: using the Intercepting system may result in unhandled errors and bugs.
 

## License
MIT
