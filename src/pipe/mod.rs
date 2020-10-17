mod fn_pipes;
pub use fn_pipes::{ MapPipe, FilterPipe, FlattenPipe };

mod merge_pipe;
pub use merge_pipe::MergePipe;

mod split_pipe;
pub use split_pipe::SplitPipe;


use std::cell::RefCell;
use std::rc::Rc;

use crate::Lattice;
use crate::merge::Merge;

/// A pipe is something which can have items added to it.
pub trait Pipe {
    type Item;

    fn merge_in(&self, input: Self::Item);
}

pub struct Tank<T, F: Merge<T>> {
    lattice: Rc<RefCell<Lattice<T, F>>>, // TODO: Rc<RefCell>.
}
impl <T, F: Merge<T>> Tank<T, F> {
    pub fn new(lattice: Lattice<T, F>) -> Self {
        Self {
            lattice: Rc::new(RefCell::new(lattice)),
        }
    }
    pub fn get_lattice(&self) -> Rc<RefCell<Lattice<T, F>>> {
        self.lattice.clone() // TODO VERY SLOPPY
    }
}
impl <T, F: Merge<T>> Pipe for Tank<T, F> {
    type Item = T;

    fn merge_in(&self, input: Self::Item) {
        self.lattice.borrow_mut().merge_in(input);
    }
}
impl <T, F: Merge<T>> Clone for Tank<T, F> {
    fn clone(&self) -> Self {
        Self {
            lattice: self.lattice.clone()
        }
    }
}

use std::hash::Hash;
use std::collections::HashMap;

impl <K: Eq + Hash, V, VF: Merge<V>, F: Merge<HashMap<K, Lattice<V, VF>>>> Tank<HashMap<K, Lattice<V, VF>>, F> {
    pub fn kv_pipe(&self) -> impl Pipe<Item = ( K, Lattice<V, VF> )> {
        MapPipe::new(self.clone(), |( k, v )| {
            // let mut y: HashMap<K, Lattice<V, VF>>
            let mut y: HashMap<_, _> = Default::default();
            y.insert(k, v);
            y
        })
    }
}
// pub struct KvPipe<'a, K: Eq + Hash, V, VF: Merge<V>, F: Merge<HashMap<K, Lattice<V, VF>>>> {
//     tank: &'a Tank<HashMap<K, Lattice<V, VF>>, F>,
// }
// impl <'a, K: Eq + Hash, V, VF: Merge<V>, F: Merge<HashMap<K, Lattice<V, VF>>>> Pipe for Tank<HashMap<K, Lattice<V, VF>>, F> {
//     type Item = (K, Lattice<V, VF>);

//     fn merge_in(&self, input: Self::Item) {
//         let mut y: HashMap<&'static str, VersionedString> = Default::default();
//         y.insert(k, (t.into(), v.into()).into());
//         y

//         self.tank.borrow_mut().merge_in
//     }
// }