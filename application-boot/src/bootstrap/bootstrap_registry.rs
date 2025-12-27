pub trait BootstrapRegistry {
    fn register<T: Send + Sync + 'static>(&self, state: T) -> bool;

    fn is_registered<T: Send + Sync + 'static>(&self) -> bool;

    fn register_if_absent<T: Send + Sync + 'static>(&self, state: T);
}
