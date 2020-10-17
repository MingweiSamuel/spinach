use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::Lattice;
use crate::merge::Merge;

use super::{ Pipe, MapPipe };


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
