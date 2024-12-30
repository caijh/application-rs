# application-rs
application-rs is framework like spring bot, write by rust lang, support develop micro service faster.

## Usage
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let application = RustApplication::default();

    application
        .add_listener(Box::new(ApplicationContextInitializedListener {}))
        .await;
    application
        .add_listener(Box::new(ApplicationStartedEventListener {}))
        .await;
    application.run().await?;

    Ok(())
}

```
