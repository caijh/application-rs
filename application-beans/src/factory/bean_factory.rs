use state::TypeMap;

pub trait BeanFactory {
    fn get<T: 'static>(&self) -> &T;
    fn try_get<T: 'static>(&self) -> Option<&T>;
}

pub trait ListableBeanFactory: BeanFactory {

    fn get_bean_definition_count(&self) -> usize;
}

pub trait ConfigurableBeanFactory {
    fn set<T: Send + Sync + 'static>(&self, state: T) -> bool;
}

#[derive(Default)]
pub struct DefaultListableBeanFactory {
    beans: TypeMap![Send + Sync],
}

impl BeanFactory for DefaultListableBeanFactory {
    fn get<T: 'static>(&self) -> &T {
        self.beans.get::<T>()
    }

    fn try_get<T: 'static>(&self) -> Option<&T> {
        self.beans.try_get::<T>()
    }
}

impl ListableBeanFactory for DefaultListableBeanFactory {
    fn get_bean_definition_count(&self) -> usize {
        self.beans.len()
    }
}

impl ConfigurableBeanFactory for DefaultListableBeanFactory {
    fn set<T: Send + Sync + 'static>(&self, state: T) -> bool {
        self.beans.set(state)
    }
}
