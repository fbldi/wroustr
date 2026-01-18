use std::pin::Pin;
use std::sync::Arc;
use uuid::Uuid;
use crate::routes::{Dispatcher, State};

//The interceptor can modify the raw incoming msg without processing it (could be)
//IMPORTANT: the interceptor can't modify ws! it receives a raw string and can process it
//this process can be done on incoming msg, and on outgoing msg!


//this may continue the routing or return.
#[derive(PartialEq)]
pub enum InterceptorResult {
    Pass(String),
    Cancel
}

#[derive(PartialEq)]
pub enum InterceptorType {
    INCOMING,
    OUTGOING,
}

pub struct ServerInterceptor<S> {
    pub r#type: InterceptorType,
    pub callback: Arc<dyn Fn(String, Uuid, State<S>) -> Pin<Box<dyn Future<Output=InterceptorResult> + Send>> + Send + Sync + 'static>
}

impl<S> ServerInterceptor<S> {
    pub fn new<F,Fut>(callback: F, r#type: InterceptorType) -> Self
    where F: Fn(String, Uuid, State<S>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output=InterceptorResult> + Send + 'static {
        ServerInterceptor { r#type, callback: Arc::new(move |incoming, uuid, state| {Box::pin(callback(incoming, uuid, state))}) }
    }
}

pub struct Interceptor<S> {
    pub r#type: InterceptorType,
    pub callback: Arc<dyn Fn(String, State<S>) -> Pin<Box<dyn Future<Output=InterceptorResult> + Send>> + Send + Sync + 'static>
}

impl<S> Interceptor<S> {
    pub fn new<F,Fut>(callback: F, r#type: InterceptorType) -> Self
    where F: Fn(String, State<S>) -> Fut + Send + Sync + 'static,
          Fut: Future<Output=InterceptorResult> + Send + 'static {
        Interceptor { r#type, callback: Arc::new(move |incoming, state| {Box::pin(callback(incoming, state))}) }
    }
}
