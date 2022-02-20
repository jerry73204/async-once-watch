# async-watch-once

\[ [crates.io](crates.io/crates/async-once-watch) | [docs.rs](https://docs.rs/async-once-watch/) \]

This crate provides a shareable container `OnceWatch<T>` in Rust, which value is set once. The readers wait in asynchronous manner until the value is ready.

```rust
use async_once_watch::OnceWatch;
use async_std::task::{sleep, spawn};
use once_cell::sync::Lazy;
use std::time::Duration;

static STATE: Lazy<OnceWatch<u8>> = Lazy::new(OnceWatch::new);
let secret = 10;

/* Writer */
spawn(async move {
    sleep(Duration::from_millis(500)).await;

    // First write is fine
    let ok = STATE.set(secret).is_ok();
    assert!(ok);

    // Second write is not allowed
    let ok = STATE.set(secret).is_ok();
    assert!(!ok);
});

/* Reader */
spawn(async move {
    let received = *STATE.get().await;
    assert_eq!(received, secret);
});
```

## License

MIT license. See [license file](LICENSE.txt).
