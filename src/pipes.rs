// use std::cell::RefCell;
use std::fmt::Debug;

// use tokio::sync::mpsc;
// use tokio::sync::broadcast;

// use tokio::stream::Stream;

use crate::merge::Merge;
// use crate::semilattice::Semilattice;


pub trait Pipe<'item> {
    type Item: 'item;

    #[must_use]
    fn push(&mut self, item: Self::Item) -> Result<(), &'static str>;
}


// pub struct DebugPipe<'item, P: Pipe<'item>>
// where
//     P::Item: Debug,
// {
//     next_pipe: P,
//     _phantom: std::marker::PhantomData<dyn Fn(&'item ())>,
// }
// impl <'item, P: Pipe<'item>> DebugPipe<'item, P>
// where
//     P::Item: Debug,
// {
//     pub fn new(next_pipe: P) -> Self {
//         Self {
//             next_pipe: next_pipe,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
// impl <'item, P: Pipe<'item>> Pipe<'item> for DebugPipe<'item, P>
// where
//     P::Item: Debug,
// {
//     type Item = P::Item;

//     fn push(&mut self, item: P::Item) -> Result<(), &'static str> {
//         println!("{:?}", item);
//         self.next_pipe.push(item)
//     }
// }


pub struct NullPipe<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T> NullPipe<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<'item, T: 'item> Pipe<'item> for NullPipe<T> {
    type Item = T;

    fn push(&mut self, _item: T) -> Result<(), &'static str> {
        Ok(())
    }
}

pub struct RefPipe<T, P: for<'item> Pipe<'item, Item = &'item T>> {
    next_pipe: P,
    _phantom: std::marker::PhantomData<T>,
}
impl<T, P: for<'item> Pipe<'item, Item = &'item T>> RefPipe<T, P> {
    pub fn new(next_pipe: P) -> Self {
        Self {
            next_pipe: next_pipe,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<'item, T: 'item, P: for<'a> Pipe<'a, Item = &'a T>> Pipe<'item> for RefPipe<T, P> {
    type Item = T;

    fn push(&mut self, item: T) -> Result<(), &'static str> {
        self.next_pipe.push(&item)
    }
}


// pub struct ClonePipe<'item, P: Pipe<'item>> {
//     next_pipe: P,
//     _phantom: std::marker::PhantomData<&'item ()>,
// }
// impl <'item, P: Pipe<'item>> ClonePipe<'item, P> {
//     pub fn new(next_pipe: P) -> Self {
//         Self {
//             next_pipe: next_pipe,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
// impl <'item, P: Pipe<'item>> Pipe<'item> for ClonePipe<'item, P>
// where
//     P::Item: Clone,
// {
//     type Item = &'item P::Item;

//     fn push(&mut self, item: Self::Item) -> Result<(), &'static str> {
//         self.next_pipe.push(item.clone())
//     }
// }

// // struct DebugPipe2<'a, P: Pipe<'a>> {
// //     next_pipe: P,
// //     _phantom: std::marker::PhantomData<&'a ()>,
// // }
// // impl<'a, P: Pipe<'a>> Pipe<'a> for DebugPipe2<'a, P>
// // where
// //     P::Item: 'a + Debug,
// // {
// //     type Item = &'a P::Item;

// //     fn push(&mut self, value: &'a P::Item) -> Result<(), &'static str> {
// //         println!("{:?}", *value);
// //         Ok(())
// //     }
// // }


// pub struct LatticePipe<'item, F: Merge, P: for<'a> Pipe<'a, Item = &'a F::Domain>> {
//     value: F::Domain,
//     next_pipe: P,
//     _phantom: std::marker::PhantomData<&'item ()>,
// }
// impl<'item, F: Merge, P: for<'a> Pipe<'a, Item = &'a F::Domain>> LatticePipe<'item, F, P> {
//     pub fn new(bottom: F::Domain, next_pipe: P) -> Self {
//         Self {
//             value: bottom,
//             next_pipe: next_pipe,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
// impl<'item, F: Merge, P: for<'a> Pipe<'a, Item = &'a F::Domain>> Pipe<'item> for LatticePipe<'item, F, P>
// where
//     F::Domain: 'item,
// {
//     type Item = F::Domain;

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



// pub struct FilterPipe<T, F, P: Pipe<T>>
// where
//     F: for<'a> UnaryFn<'a, &'a T, bool>,
// {
//     next_pipe: P,
//     _phantom: std::marker::PhantomData<( T, F )>,
// }
// impl <T, F, P: Pipe<T>> FilterPipe<T, F, P>
// where
//     F: for<'a> UnaryFn<'a, &'a T, bool>,
// {
//     pub fn new(next_pipe: P) -> Self {
//         Self {
//             next_pipe: next_pipe,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
// impl <T, F, P: Pipe<T>> Pipe<T> for FilterPipe<T, F, P>
// where
//     F: for<'a> UnaryFn<'a, &'a T, bool>,
// {
//     fn push(&mut self, item: T) -> Result<(), &'static str> {
//         if F::call(&item) {
//             self.next_pipe.push(item)
//         }
//         else {
//             Ok(())
//         }
//     }
// }



// pub struct MapPipe<'a, I, O, F: UnaryFn<'a, I, O>, P: Pipe<O>> {
//     next_pipe: P,
//     _phantom: std::marker::PhantomData<&'a ( I, O, F )>,
// }
// impl <'a, I, O, F: UnaryFn<'a, I, O>, P: Pipe<O>> MapPipe<'a, I, O, F, P> {
//     pub fn new(next_pipe: P) -> Self {
//         Self {
//             next_pipe: next_pipe,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
// impl <'a, I, O, F: UnaryFn<'a, I, O>, P: Pipe<O>> Pipe<I> for MapPipe<'a, I, O, F, P> {
//     fn push(&mut self, item: I) -> Result<(), &'static str> {
//         self.next_pipe.push(F::call(item))
//     }
// }



#[test]
pub fn test_stuff() {
    use std::collections::HashMap;
    use crate::merge::{ MapUnion, Max };

    type Asdf = impl for<'item> Pipe<'item, Item = &'item usize>;
    let pipe: Asdf = NullPipe::new();
    // let pipe = RefPipe::<usize, NullPipe<&'_ usize>>::new(pipe);
    // let pipe = LatticePipe::<'_, MapUnion<HashMap<usize, Max<usize>>>, _>::new(HashMap::new(), pipe);
    // pipe.push(5_usize);
    // pipe.push(10_usize);
}


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

//     // struct GetHello2;
//     // impl UnaryFn<HashMap<&'static str, &'static str>, Option<String>> for GetHello2 {
//     //     fn call(input: HashMap<&'static str, &'static str>) -> Option<String> {
//     //         input.get("hello").map(|x| (*x).to_owned())
//     //     }
//     // }

//     // struct GetHelloPipe<P: Pipe<String>> {
//     //     next_pipe: P,
//     // }
//     // impl <P: Pipe<String>> GetHelloPipe<P> {
//     //     pub fn new(next_pipe: P) -> Self {
//     //         Self {
//     //             next_pipe: next_pipe,
//     //         }
//     //     }
//     // }
//     // impl <'a, P: Pipe<String>> Pipe<&'a HashMap<&'static str, &'static str>> for GetHelloPipe<P> {
//     //     fn push(&mut self, input: &'a HashMap<&'static str, &'static str>) -> Result<(), &'static str> {
//     //         match input.get("hello") {
//     //             Some(val) => self.next_pipe.push((*val).to_owned()),
//     //             None => Ok(()),
//     //         }
//     //     }
//     // }

//     let pipe: NullPipe = NullPipe;
//     let pipe = DebugPipe::new(pipe);
//     // let pipe: ClonePipe<_, _> = ClonePipe::new(pipe);


//     // type RefPipe = for<'a> Pipe<&'a HashMap<&'static str, &'static str>>;
//     let pipe = MapPipe::<_, _, GetHello, _>::new(pipe);


//     // let pipe = MapPipe::<HashMap<&'static str, &'static str>, Option<String>, GetHello2, _>::new(pipe);
//     // let pipe: ClonePipe<_, _> = ClonePipe::new(pipe);

//     // let pipe: GetHelloPipe<_> = GetHelloPipe::new(pipe);


//     let pipe: LatticePipe::<MapUnion<HashMap<&'static str, Max<&'static str>>>, _> = LatticePipe::new(Default::default(), pipe);
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
