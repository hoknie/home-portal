use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

pub type Pending<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub async fn join_all<T>(mut futures: Vec<Pending<'_, T>>) -> Vec<T> {
    let mut results: Vec<Option<T>> = futures.iter().map(|_| None).collect();
    std::future::poll_fn(|context| {
        let mut waiting = false;
        for (future, result) in futures.iter_mut().zip(results.iter_mut()) {
            if result.is_none() {
                match future.as_mut().poll(context) {
                    Poll::Ready(value) => *result = Some(value),
                    Poll::Pending => waiting = true,
                }
            }
        }
        if waiting {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    })
    .await;
    results.into_iter().flatten().collect()
}
