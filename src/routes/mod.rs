use std::collections::HashMap;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

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