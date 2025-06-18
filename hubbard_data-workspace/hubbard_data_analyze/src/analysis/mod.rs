pub mod channel_view;
pub mod total_view;

/// Provides higher order functions for extensibility
pub trait Functor {
    fn map<U, F: FnMut(Self) -> U>(self, mut f: F) -> U
    where
        Self: std::marker::Sized,
    {
        f(self)
    }
    fn map_ref<U, F: FnMut(&Self) -> U>(&self, mut f: F) -> U {
        f(self)
    }
    fn map_mut<U, F: FnMut(&mut Self)>(&mut self, mut f: F) {
        f(self)
    }
}
