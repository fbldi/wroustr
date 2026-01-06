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
        });
    
        server.route("CONNECTED", |params, disp, state|async move {
            println!("New connection: {}", params.get("uuid").unwrap());
            println!("State: {:?}", state);
            disp.send("@CONNECTION-ESTABLISHED");
        });
    
        server.route("@INIT", init);
    
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


## Parameters
all parameters will be parsed as strings. the keys should always begin with `#`
and the value must be separated with a space.

on the server, there's always an uuid parameter to keep track of the client.

## Appstate
the state is passed to all routes, but by default it's immutable.
in order to create mutable states, use `Mutex`, `Atomic*`, `DashMap`, etc. as fields.

## License
MIT