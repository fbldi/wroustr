use std::pin::Pin;
use std::sync::Arc;
use crate::routes::{Dispatcher, Params, State};

pub struct Layer<S> {
    pub name: String,
    pub(crate) allowed: Vec<String>,
    pub(crate) blocked: Vec<String>,
    pub callback: Arc<dyn Fn(Params, Dispatcher, State<S>) -> Pin<Box<dyn Future<Output=bool> + Send>> + Send + Sync + 'static>
}

impl<S> Clone for Layer<S> {
    fn clone(&self) -> Self {
        self.clone()
    }
}

impl<S> Layer<S> {
    pub fn new<F, Fut>(name: impl Into<String>, callback: F) -> Layer<S>
    where
        F: Fn(Params, Dispatcher, State<S>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = bool> + Send + 'static,{
        Self {
            name: name.into(),
            allowed: Vec::new(),
            blocked: Vec::new(),
            callback: Arc::new(move |params, dispatcher, state| {
                Box::pin(callback(params, dispatcher, state))
            }),
        }
    }
    pub fn allow(mut self, allow: Vec<impl Into<String>>) -> Layer<S> {
        self.allowed = allow.into_iter().map(|x| x.into()).collect();
        self
    }
    pub fn block(mut self, blocked: Vec<impl Into<String>>) -> Layer<S> {
        self.blocked = blocked.into_iter().map(|x| x.into()).collect();
        self
    }
}