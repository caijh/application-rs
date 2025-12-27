pub trait BootstrapContext {
    fn get<T: Send + Sync + 'static>(&self) -> Option<&T>;
    fn is_registered<T: Send + Sync + 'static>(&self) -> bool;
}
