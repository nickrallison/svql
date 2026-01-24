use std::marker::PhantomData;

/// A logical connection point on a Pattern component.
#[derive(Debug, Clone)]
pub struct Wire<P, S> {
    pub name: String,
    _marker: PhantomData<(P, S)>,
}

impl<P, S> Wire<P, S> {
    pub fn new<T>(name: impl ToString, _unused: T) -> Self {
        Self {
            name: name.to_string(),
            _marker: PhantomData,
        }
    }
}
