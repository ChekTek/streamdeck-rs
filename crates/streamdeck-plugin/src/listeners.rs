use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use futures_util::future::BoxFuture;

pub type Cb<T> = Arc<dyn Fn(T) -> BoxFuture<'static, ()> + Send + Sync>;

pub struct ListenerSet<T> {
    next_id: AtomicU64,
    items: RwLock<Vec<(u64, Cb<T>)>>,
}

impl<T> ListenerSet<T> {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            items: RwLock::new(Vec::new()),
        }
    }

    pub fn add(&self, cb: Cb<T>) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.items.write().expect("listener set").push((id, cb));
        id
    }

    pub fn remove(&self, id: u64) {
        self.items
            .write()
            .expect("listener set")
            .retain(|(item_id, _)| *item_id != id);
    }

    pub fn snapshot(&self) -> Vec<Cb<T>> {
        self.items
            .read()
            .expect("listener set")
            .iter()
            .map(|(_, cb)| cb.clone())
            .collect()
    }
}

impl<T> Default for ListenerSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + 'static> ListenerSet<T> {
    pub async fn emit(&self, event: T) {
        for cb in self.snapshot() {
            cb(event.clone()).await;
        }
    }
}

/// Handle that can unsubscribe a listener. Dropping this does **not** unsubscribe
/// (matching the TypeScript `IDisposable` which must be disposed explicitly).
#[derive(Clone)]
pub struct Subscription {
    unsub: Arc<dyn Fn() + Send + Sync>,
}

impl Subscription {
    pub fn new(unsub: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            unsub: Arc::new(unsub),
        }
    }

    pub fn unsubscribe(&self) {
        (self.unsub)();
    }
}

pub fn subscribe<T, F, Fut>(set: &Arc<ListenerSet<T>>, listener: F) -> Subscription
where
    T: Clone + Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let listener = Arc::new(listener);
    let cb: Cb<T> = Arc::new(move |ev| {
        let listener = listener.clone();
        Box::pin(async move { listener(ev).await })
    });
    let id = set.add(cb);
    let set = set.clone();
    Subscription::new(move || set.remove(id))
}
