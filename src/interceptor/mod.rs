use std::pin::Pin;
use std::sync::Arc;
use crate::routes::{Dispatcher};

//The interceptor can modify the raw incoming msg without processing it (could be)
//IMPORTANT: the interceptor MUST handle ws! Give it a Dispathcer!


//this may continue the routing or return.
#[derive(PartialEq)]
pub enum InterceptorResult {
    Pass(String),
    Cancel
}

pub struct Interceptor {
    pub callback: Arc<dyn Fn(String, Dispatcher) -> Pin<Box<dyn Future<Output=InterceptorResult> + Send>> + Send + Sync + 'static>
}

impl Interceptor {
    pub fn new<F,Fut>(callback: F) -> Self
    where F: Fn(String, Dispatcher) -> Fut + Send + Sync + 'static,
        Fut: Future<Output=InterceptorResult> + Send + 'static {
        Interceptor { callback: Arc::new(move |incoming, dp| {Box::pin(callback(incoming, dp))}) }
    }
}