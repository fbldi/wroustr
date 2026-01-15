use std::pin::Pin;
use std::sync::Arc;
use crate::routes::{Dispatcher};

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

pub struct Interceptor {
    pub r#type: InterceptorType,
    pub callback: Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output=InterceptorResult> + Send>> + Send + Sync + 'static>
}

impl Interceptor {
    pub fn new<F,Fut>(callback: F, r#type: InterceptorType) -> Self
    where F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: Future<Output=InterceptorResult> + Send + 'static {
        Interceptor { r#type, callback: Arc::new(move |incoming| {Box::pin(callback(incoming))}) }
    }
}