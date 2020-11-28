use futures::{stream::{FuturesUnordered, Stream}, future::{Future, FutureExt, BoxFuture}};
use std::{pin::Pin, task::{Context, Poll}, sync::{Arc, Mutex}};
use pin_project_lite::pin_project;

pub struct Spawnner {
    tasks: Arc<Mutex<FuturesUnordered::<BoxFuture<'static, ()>>>>,
}
impl Spawnner {
    pub fn spawn<T>(&self, task: T)
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let fut = async {
            task.await;
        };
        self.tasks.lock().unwrap().push(fut.boxed());
    }
}

pin_project! {
    struct Wrapper<'a, Fut>
    where
        Fut: Future,
    {
        #[pin]
        fut: Fut,
        #[pin]
        tasks: &'a Mutex<FuturesUnordered::<BoxFuture<'static, ()>>>,
    }
}

impl<'a, Fut> Future for Wrapper<'a, Fut>
where
    Fut: Future,
{
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut self.project();
        loop {
            if let Poll::Ready(r) = Pin::new(&mut this.fut).poll(cx) {
                return Poll::Ready(r)
            }

            let tasks = &mut *this.tasks.lock().unwrap();
            if tasks.len() == 0 {
                return Poll::Pending
            }

            if let Poll::Pending = Pin::new(tasks).poll_next(cx) {
                return Poll::Pending
            }
        }
    }
}

pub async fn with_spawn<F, Fut>(f: F) -> Fut::Output
where
    F: FnOnce(Spawnner) -> Fut,
    Fut: Future + Send,
{
    let tasks = Arc::new(Mutex::new(FuturesUnordered::new()));
    let spawnner = Spawnner {
        tasks: tasks.clone(),
    };
    let fut = f(spawnner);

    Wrapper {
        fut,
        tasks: &tasks,
    }.await
}

#[tokio::test]
async fn test() {
    use tokio::time::{sleep, Duration};
    with_spawn(|spawnner| async move {
        spawnner.spawn(async {
            sleep(Duration::from_secs(2)).await;
            println!("2");
        });
        sleep(Duration::from_secs(5)).await;
        println!("5");
    }).await;
}
