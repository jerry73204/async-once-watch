use async_once_watch::OnceWatch;
use async_std::task::{sleep, spawn};
use futures::future;
use once_cell::sync::Lazy;
use rand::prelude::*;
use std::time::Duration;

#[async_std::test]
async fn pubsub_test() {
    static STATE: Lazy<OnceWatch<u8>> = Lazy::new(OnceWatch::new);

    let secret: u8 = rand::thread_rng().gen();

    let producer = spawn(async move {
        sleep(Duration::from_millis(200)).await;

        assert!(STATE.set(secret).is_ok());
        assert_eq!(STATE.set(secret), Err(secret));
    });

    let fast_consumers = (0..16).map(|_| {
        spawn(async move {
            assert!(STATE.try_get().is_none());

            let received = *STATE.get().await;
            assert_eq!(received, secret);
        })
    });

    let slow_consumers = (0..16).map(|_| {
        spawn(async move {
            sleep(Duration::from_millis(500)).await;

            let received = *STATE.get().await;
            assert_eq!(received, secret);
        })
    });

    futures::join!(
        producer,
        future::join_all(fast_consumers),
        future::join_all(slow_consumers)
    );
}
