use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

pub type Pending<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub async fn join_in_order<T>(mut futures: Vec<Pending<'_, T>>) -> (Vec<T>, Vec<usize>) {
    let mut results: Vec<Option<T>> = futures.iter().map(|_| None).collect();
    let mut order = Vec::with_capacity(futures.len());
    std::future::poll_fn(|context| {
        let mut waiting = false;
        for (index, (future, result)) in futures.iter_mut().zip(results.iter_mut()).enumerate() {
            if result.is_none() {
                match future.as_mut().poll(context) {
                    Poll::Ready(value) => {
                        *result = Some(value);
                        order.push(index);
                    }
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
    (results.into_iter().flatten().collect(), order)
}
