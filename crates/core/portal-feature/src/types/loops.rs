use std::future::Future;
use std::pin::Pin;

pub type Loop = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
