use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::server::ServerDispatcher as ServerDispatcher;

pub type State<S> = Arc<S>;

pub struct Route<S>
{
    pub(crate) name: String, //@NAME
    pub(crate) callback: Arc<dyn Fn(Params, Dispatcher, State<S>) -> Pin<Box<dyn Future<Output=()> + Send>> + Send + Sync + 'static>
}

pub struct ServerRoutes<S>
{
    pub(crate) name: String, //@NAME
    pub(crate) callback: Arc<dyn Fn(Params, ServerDispatcher, State<S>) -> Pin<Box<dyn Future<Output=()> + Send>> + Send + Sync + 'static>
}

pub type Params = HashMap<String, String>;


#[derive(Clone)]
pub struct Dispatcher {
    pub(crate) sender: tokio::sync::mpsc::UnboundedSender<String>,
}


impl Dispatcher {
    pub fn send(&self, msg: impl Into<String>) {
        let _ = self.sender.send(msg.into()).unwrap();
    }

    pub async fn keep_alive(&self) {
        futures_util::future::pending::<()>().await;
    }

}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ConnectionId(pub Uuid);

pub(crate) fn alert(msg: impl Into<String>) {
    println!("{}", msg.into());
}



