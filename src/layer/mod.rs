use std::pin::Pin;
use std::sync::Arc;
use crate::routes::{Dispatcher, Params, State};
use crate::routes::ServerDispatcher;

pub enum LayerResult {
    Pass(Params),
    Cancel
}

pub struct ClientLayer<S> {
    pub name: String,
    pub(crate) allowed: Vec<String>,
    pub(crate) blocked: Vec<String>,
    pub callback: Arc<dyn Fn(Params, Dispatcher, State<S>) -> Pin<Box<dyn Future<Output=LayerResult> + Send>> + Send + Sync + 'static>
}

impl<S> Clone for ClientLayer<S> {
    fn clone(&self) -> Self {
        ClientLayer {
            name: self.name.clone(),
            allowed: self.allowed.clone(),
            blocked: self.blocked.clone(),
            callback: self.callback.clone(),
        }
    }
}

impl<S> ClientLayer<S> {
    pub fn new<F, Fut>(name: impl Into<String>, callback: F) -> ClientLayer<S>
    where
        F: Fn(Params, Dispatcher, State<S>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = LayerResult> + Send + 'static,{
        Self {
            name: name.into(),
            allowed: Vec::new(),
            blocked: Vec::new(),
            callback: Arc::new(move |params, dispatcher, state| {
                Box::pin(callback(params, dispatcher, state))
            }),
        }
    }
    pub fn allow(mut self, allow: Vec<impl Into<String>>) -> ClientLayer<S> {
        self.allowed = allow.into_iter().map(|x| x.into()).collect();
        self
    }
    pub fn block(mut self, blocked: Vec<impl Into<String>>) -> ClientLayer<S> {
        self.blocked = blocked.into_iter().map(|x| x.into()).collect();
        self
    }
}


pub struct ServerLayer<S> {
    pub name: String,
    pub(crate) allowed: Vec<String>,
    pub(crate) blocked: Vec<String>,
    pub callback: Arc<dyn Fn(Params, ServerDispatcher, State<S>) -> Pin<Box<dyn Future<Output=LayerResult> + Send>> + Send + Sync + 'static>
}

impl<S> Clone for ServerLayer<S> {
    fn clone(&self) -> Self {
        ServerLayer {
            name: self.name.clone(),
            allowed: self.allowed.clone(),
            blocked: self.blocked.clone(),
            callback: self.callback.clone(),
        }
    }
}

impl<S> ServerLayer<S> {
    pub fn new<F, Fut>(name: impl Into<String>, callback: F) -> ServerLayer<S>
    where
        F: Fn(Params, ServerDispatcher, State<S>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = LayerResult> + Send + 'static,{
        Self {
            name: name.into(),
            allowed: Vec::new(),
            blocked: Vec::new(),
            callback: Arc::new(move |params, dispatcher, state| {
                Box::pin(callback(params, dispatcher, state))
            }),
        }
    }
    pub fn allow(mut self, allow: Vec<impl Into<String>>) -> ServerLayer<S> {
        self.allowed = allow.into_iter().map(|x| x.into()).collect();
        self
    }
    pub fn block(mut self, blocked: Vec<impl Into<String>>) -> ServerLayer<S> {
        self.blocked = blocked.into_iter().map(|x| x.into()).collect();
        self
    }
}