// use std::cell::RefCell;
use std::fmt::Debug;

// use tokio::sync::mpsc;
// use tokio::sync::broadcast;

// use tokio::stream::Stream;

use crate::merge::Merge;
// use crate::semilattice::Semilattice;


pub trait Pipe<T> {
    #[must_use]
    fn push(&mut self, item: T) -> Result<(), &'static str>;
}


pub struct DebugPipe<T: Debug, P: Pipe<T>> {
    next_pipe: P,
    _phantom: std::marker::PhantomData<T>,
}
impl <T: Debug, P: Pipe<T>> DebugPipe<T, P> {
    pub fn new(next_pipe: P) -> Self {
        Self {
            next_pipe: next_pipe,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl <T: Debug, P: Pipe<T>> Pipe<T> for DebugPipe<T, P> {
    fn push(&mut self, item: T) -> Result<(), &'static str> {
        println!("{:?}", item);
        self.next_pipe.push(item)
    }
}


pub struct NullPipe;
impl <T> Pipe<T> for NullPipe {
    fn push(&mut self, _item: T) -> Result<(), &'static str> {
        Ok(())
    }
}


pub struct ClonePipe<T, P: Pipe<T>> {
    next_pipe: P,
    _phantom: std::marker::PhantomData<T>,
}
impl <T, P: Pipe<T>> ClonePipe<T, P> {
    pub fn new(next_pipe: P) -> Self {
        Self {
            next_pipe: next_pipe,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl <'a, T: Clone, P: Pipe<T>> Pipe<&'a T> for ClonePipe<T, P> {
    fn push(&mut self, item: &'a T) -> Result<(), &'static str> {
        self.next_pipe.push(item.clone())
    }
}


pub struct LatticePipe<F: Merge, P: for<'a> Pipe<&'a F::Domain>> {
    value: F::Domain,
    next_pipe: P,
}
impl <F: Merge, P: for<'a> Pipe<&'a F::Domain>> LatticePipe<F, P> {
    pub fn new(bottom: F::Domain, next_pipe: P) -> Self {
        Self {
            value: bottom,
            next_pipe: next_pipe,
        }
    }
}
impl <F: Merge, P: for<'a> Pipe<&'a F::Domain>> Pipe<F::Domain> for LatticePipe<F, P> {
    fn push(&mut self, item: F::Domain) -> Result<(), &'static str> {
        F::merge_in(&mut self.value, item);
        self.next_pipe.push(&self.value)
    }
}


pub struct IntoPipe<T, U: From<T>, P: Pipe<U>> {
    next_pipe: P,
    _phantom: std::marker::PhantomData<( T, U )>,
}
impl <T, U: From<T>, P: Pipe<U>> IntoPipe<T, U, P> {
    pub fn new(next_pipe: P) -> Self {
        Self {
            next_pipe: next_pipe,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl <T, U: From<T>, P: Pipe<U>> Pipe<T> for IntoPipe<T, U, P> {
    fn push(&mut self, item: T) -> Result<(), &'static str> {
        self.next_pipe.push(item.into())
    }
}

use std::iter::{ IntoIterator, FromIterator };

pub struct CollectPipe<A, T: IntoIterator<Item = A>, U: FromIterator<A>, P: Pipe<U>> {
    next_pipe: P,
    _phantom: std::marker::PhantomData<( A, T, U )>,
}
impl <A, T: IntoIterator<Item = A>, U: FromIterator<A>, P: Pipe<U>> CollectPipe<A, T, U, P> {
    pub fn new(next_pipe: P) -> Self {
        Self {
            next_pipe: next_pipe,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl <A, T: IntoIterator<Item = A>, U: FromIterator<A>, P: Pipe<U>> Pipe<T> for CollectPipe<A, T, U, P> {
    fn push(&mut self, item: T) -> Result<(), &'static str> {
        self.next_pipe.push(item.into_iter().collect())
    }
}


// pub struct MapPipe<T, U, F: Fn(T) -> U> {
//     f: F,
//     _phantom: std::marker::PhantomData<( T, U )>,
// }



#[test]
pub fn test_stuff() {
    use std::collections::HashSet;

    let pipe = NullPipe;
    let pipe = DebugPipe::new(pipe);
    let pipe = ClonePipe::new(pipe);
    let pipe = LatticePipe::<crate::merge::Union<HashSet<usize>>, _>::new(Default::default(), pipe);
    let pipe = CollectPipe::new(pipe);
    let pipe = ClonePipe::new(pipe);
    let mut pipe = pipe;


    let items: Vec<Vec<usize>> = vec![ vec![ 1 ], vec![ 2 ], vec![ 3 ], vec![ 4 ], vec![ 5 ] ];

    for item in &items {
        pipe.push(item).unwrap();
    }
}
