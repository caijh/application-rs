use application_core::lang::runnable::Runnable;

pub struct Task {
    runnable: Box<dyn Runnable>,
}

impl Task {
    pub fn new(runnable: Box<dyn Runnable>) -> Self {
        Self { runnable }
    }
    pub fn get_runnable(&self) -> &Box<dyn Runnable> {
        &self.runnable
    }
}
