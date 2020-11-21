use std::borrow::Cow;
// use std::cell::RefCell;
use std::fmt::Debug;

// use tokio::sync::mpsc;
// use tokio::sync::broadcast;

// use tokio::stream::Stream;

use crate::merge::Merge;
// use crate::semilattice::Semilattice;


pub trait Pipe<T: ToOwned> {
    #[must_use]
    fn push<'item>(&mut self, item: Cow<'item, T>) -> Result<(), &'static str>
    where
        T: 'item;
}

pub struct NullPipe;
impl<T: ToOwned> Pipe<T> for NullPipe {
    fn push<'item>(&mut self, item: Cow<'item, T>) -> Result<(), &'static str>
    where
        T: 'item
    {
        std::mem::drop(item);
        Ok(())
    }
}

pub struct ClonePipe<T: ToOwned, P: Pipe<T>> {
    next_pipe: P,
    _phantom: std::marker::PhantomData<T>,
}
impl<T: ToOwned, P: Pipe<T>> ClonePipe<T, P> {
    pub fn new(next_pipe: P) -> Self {
        Self {
            next_pipe: next_pipe,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<T: ToOwned, P: Pipe<T>> Pipe<T> for ClonePipe<T, P> {
    fn push<'item>(&mut self, item: Cow<'item, T>) -> Result<(), &'static str>
    where
        T: 'item,
    {
        self.next_pipe.push(Cow::Owned(item.into_owned()))
    }
}

// pub struct RefPipe<T, P: for<'a> Pipe<&'a T>> {
//     next_pipe: P,
//     _phantom: std::marker::PhantomData<T>,
// }
// impl<T, P: for<'a> Pipe<&'a T>> RefPipe<T, P> {
//     pub fn new(next_pipe: P) -> Self {
//         Self {
//             next_pipe: next_pipe,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
// impl<T, P: for<'a> Pipe<&'a T>> Pipe<T> for RefPipe<T, P> {
//     fn push<'item>(&mut self, item: T) -> Result<(), &'static str>
//     where
//         T: 'item
//     {
//         self.next_pipe.push(&item)
//     }
// }


// pub struct DebugPipe<T: Debug, P: Pipe<T>> {
//     next_pipe: P,
//     _phantom: std::marker::PhantomData<T>,
// }
// impl <T: Debug, P: Pipe<T>> DebugPipe<T, P> {
//     pub fn new(next_pipe: P) -> Self {
//         Self {
//             next_pipe: next_pipe,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
// impl <T: Debug, P: Pipe<T>> Pipe<T> for DebugPipe<T, P> {
//     fn push<'item>(&mut self, item: T) -> Result<(), &'static str>
//     where
//         T: 'item,
//     {
//         println!("{:?}", item);
//         self.next_pipe.push(item)
//     }
// }



// fn test_stuff() {
//     let pipe = NullPipe;
//     let pipe = ClonePipe::<usize, NullPipe>::new(pipe);
//     let pipe = DebugPipe::new(pipe);
//     let pipe = RefPipe::new(pipe);
//     // let mut pipe = pipe;
//     // pipe.push(500_usize);
// }

// #[test]
// fn test() {
//     test_stuff();
// }


// pub struct NullPipe;
// impl <T> Pipe<T> for NullPipe {
//     fn push(&mut self, _item: T) -> Result<(), &'static str> {
//         Ok(())
//     }
// }


// pub struct ClonePipe<T, P: Pipe<T>> {
//     next_pipe: P,
//     _phantom: std::marker::PhantomData<T>,
// }
// impl <T, P: Pipe<T>> ClonePipe<T, P> {
//     pub fn new(next_pipe: P) -> Self {
//         Self {
//             next_pipe: next_pipe,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
// impl <'a, T: Clone, P: Pipe<T>> Pipe<&'a T> for ClonePipe<T, P> {
//     fn push(&mut self, item: &'a T) -> Result<(), &'static str> {
//         self.next_pipe.push(item.clone())
//     }
// }


// pub struct RefPipe<T, P: for<'a> Pipe<&'a T>> {
//     next_pipe: P,
//     _phantom: std::marker::PhantomData<T>,
// }
// impl <T, P: for<'a> Pipe<&'a T>> RefPipe<T, P> {
//     pub fn new(next_pipe: P) -> Self {
//         Self {
//             next_pipe: next_pipe,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
// impl <T, P: for<'a> Pipe<&'a T>> Pipe<T> for RefPipe<T, P> {
//     fn push(&mut self, item: T) -> Result<(), &'static str> {
//         self.next_pipe.push(&item)
//     }
// }


// pub struct LatticePipe<F: Merge, P: for<'a> Pipe<&'a F::Domain>> {
//     value: F::Domain,
//     next_pipe: P,
// }
// impl <F: Merge, P: for<'a> Pipe<&'a F::Domain>> LatticePipe<F, P> {
//     pub fn new(bottom: F::Domain, next_pipe: P) -> Self {
//         Self {
//             value: bottom,
//             next_pipe: next_pipe,
//         }
//     }
// }
// impl <F: Merge, P: for<'a> Pipe<&'a F::Domain>> Pipe<F::Domain> for LatticePipe<F, P> {
//     fn push(&mut self, item: F::Domain) -> Result<(), &'static str> {
//         F::merge_in(&mut self.value, item);
//         self.next_pipe.push(&self.value)
//     }
// }


// pub struct IntoPipe<T, U: From<T>, P: Pipe<U>> {
//     next_pipe: P,
//     _phantom: std::marker::PhantomData<( T, U )>,
// }
// impl <T, U: From<T>, P: Pipe<U>> IntoPipe<T, U, P> {
//     pub fn new(next_pipe: P) -> Self {
//         Self {
//             next_pipe: next_pipe,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
// impl <T, U: From<T>, P: Pipe<U>> Pipe<T> for IntoPipe<T, U, P> {
//     fn push(&mut self, item: T) -> Result<(), &'static str> {
//         self.next_pipe.push(item.into())
//     }
// }

// use std::iter::{ IntoIterator, FromIterator };

// pub struct CollectPipe<A, T: IntoIterator<Item = A>, U: FromIterator<A>, P: Pipe<U>> {
//     next_pipe: P,
//     _phantom: std::marker::PhantomData<( A, T, U )>,
// }
// impl <A, T: IntoIterator<Item = A>, U: FromIterator<A>, P: Pipe<U>> CollectPipe<A, T, U, P> {
//     pub fn new(next_pipe: P) -> Self {
//         Self {
//             next_pipe: next_pipe,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
// impl <A, T: IntoIterator<Item = A>, U: FromIterator<A>, P: Pipe<U>> Pipe<T> for CollectPipe<A, T, U, P> {
//     fn push(&mut self, item: T) -> Result<(), &'static str> {
//         self.next_pipe.push(item.into_iter().collect())
//     }
// }


// pub trait UnaryFn<'a, I, O> {
//     // type Input;
//     // type Output;

//     fn call(input: I) -> O;
// }



// // pub struct FilterPipe<T, F, P: Pipe<T>>
// // where
// //     F: for<'a> UnaryFn<&'a T, bool>,
// // {
// //     next_pipe: P,
// //     _phantom: std::marker::PhantomData<( T, F )>,
// // }
// // impl <T, F, P: Pipe<T>> FilterPipe<T, F, P>
// // where
// //     F: for<'a> UnaryFn<&'a T, bool>,
// // {
// //     pub fn new(next_pipe: P) -> Self {
// //         Self {
// //             next_pipe: next_pipe,
// //             _phantom: std::marker::PhantomData,
// //         }
// //     }
// // }
// // impl <T, F, P: Pipe<T>> Pipe<T> for FilterPipe<T, F, P>
// // where
// //     F: for<'a> UnaryFn<&'a T, bool>,
// // {
// //     fn push(&mut self, item: T) -> Result<(), &'static str> {
// //         if F::call(&item) {
// //             self.next_pipe.push(item)
// //         }
// //         else {
// //             Ok(())
// //         }
// //     }
// // }



// pub struct MapPipe<I, O, F: for<'a> UnaryFn<'a, I, O>, P: Pipe<O>> {
//     next_pipe: P,
//     _phantom: std::marker::PhantomData<( I, O, F )>,
// }
// impl <I, O, F: for<'a> UnaryFn<'a, I, O>, P: Pipe<O>> MapPipe<I, O, F, P> {
//     pub fn new(next_pipe: P) -> Self {
//         Self {
//             next_pipe: next_pipe,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
// impl <I, O, F: for<'a> UnaryFn<'a, I, O>, P: Pipe<O>> Pipe<I> for MapPipe<I, O, F, P> {
//     fn push(&mut self, item: I) -> Result<(), &'static str> {
//         self.next_pipe.push(F::call(item))
//     }
// }





// #[test]
// pub fn test_stuff() {
//     use crate::merge::{ MapUnion, Max };
//     use std::collections::HashMap;

//     struct GetHello;
//     impl <'a> UnaryFn<'a, &'a HashMap<&'static str, &'static str>, Option<String>> for GetHello {
//         fn call(input: &'a HashMap<&'static str, &'static str>) -> Option<String> {
//             input.get("hello").map(|x| (*x).to_owned())
//         }
//     }

//     let pipe: NullPipe = NullPipe;
//     let pipe = DebugPipe::new(pipe);
//     let pipe = MapPipe::<_, _, GetHello, _>::new(pipe);
//     let pipe = RefPipe::new(pipe);
//     let mut pipe = pipe;

//     {
//         // let hashMap = vec![
//         //     ( "hello", "world" ),
//         //     ( "foo", "bar" ),
//         // ].into_iter().collect();

//         // pipe.push(hashMap);
//     }

//     // let pipe: LatticePipe::<MapUnion<HashMap<&'static str, Max<&'static str>>>, _> = LatticePipe::new(Default::default(), pipe);
//     // let pipe = CollectPipe::new(pipe);
//     // let pipe = ClonePipe::new(pipe);
//     // let mut pipe = pipe;


//     // let items: Vec<Vec<_>> = vec![
//     //     vec![ ( "hello", "peeps" ) ],
//     //     vec![ ( "foo", "bar" ) ],
//     //     vec![ ( "bang", "baz" ) ],
//     //     vec![ ( "hello", "world" ) ],
//     //     vec![ ( "ding", "dong" ) ],
//     // ];

//     // for item in &items {
//     //     pipe.push(item).unwrap();
//     // }
// }
