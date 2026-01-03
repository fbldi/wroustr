use std::collections::HashMap;

pub struct Route {
    pub(crate) name: String, //@NAME
    pub(crate) callback: Box<dyn Fn(&Params, &Dispatcher) + Send + Sync + 'static>
}

pub type Params = HashMap<String, String>;



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