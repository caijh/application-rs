pub trait StartupStep: Send + Sync {
    fn get_name(&self) -> String;
    fn get_id(&self) -> u64;
    fn get_parent_id(&self) -> Option<u64>;
    fn get_tags(&self) -> Vec<Tag>;
    fn end(&self);
}

pub struct Tag {
    key: String,
    value: String,
}
