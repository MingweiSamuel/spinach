use std::array::IntoIter;
use std::collections::{BTreeMap, HashMap};
//use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

// fn bool_to_option<'a>(value: bool) -> Option<&'a ()> {
//     if value { Some(&()) } else { None }
// }

// fn bool_to_option_mut<'a>(value: bool) -> Option<&'a mut ()> {
//     if value {
//         Some(&mut ())
//     } 
//     else {
//         None
//     }
// }

pub trait Dict<K, V> {
    fn get(&self, key: &K) -> Option<&V>;
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;
}

// impl<K: Eq + Hash> Dict<K, ()> for HashSet<K> {
//     fn get(&self, key: &K) -> Option<&()> {
//         bool_to_option(self.contains(key))
//     }
//     fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
//         bool_to_option_mut(self.contains(key))
//     }
// }

// impl<K: Eq + Ord> Dict<K, ()> for BTreeSet<K> {
//     fn get(&self, key: &K) -> Option<&()> {
//         bool_to_option(self.contains(key))
//     }
//     fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
//         bool_to_option_mut(self.contains(key))
//     }
// }

// impl<K: Eq> Dict<K, ()> for Vec<K> {
//     fn get(&self, key: &K) -> Option<&()> {
//         bool_to_option(<[K]>::contains(self, key))
//     }
//     fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
//         bool_to_option_mut(self.contains(key))
//     }
// }

// impl<K: Eq, const N: usize> Dict<K, ()> for Array<K, N> {
//     fn get(&self, key: &K) -> Option<&()> {
//         bool_to_option(self.0.contains(key))
//     }
//     fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
//         bool_to_option_mut(self.0.contains(key))
//     }
// }

// impl<K: Eq, const N: usize> Dict<K, ()> for MaskedArray<K, N> {
//     fn get(&self, key: &K) -> Option<&()> {
//         bool_to_option(self.mask.iter()
//                 .zip(self.vals.iter())
//                 .any(|(mask, item)| *mask && item == key))
//     }
//     fn get_mut(&mut self, key: &K) -> Option<&mut ()> {
//         bool_to_option_mut(self.mask.iter()
//                 .zip(self.vals.iter())
//                 .any(|(mask, item)| *mask && item == key))
//     }
// }




impl<K: Eq + Hash, V> Dict<K, V> for HashMap<K, V> {
    fn get(&self, key: &K) -> Option<&V> {
        self.get(key)
    }
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.get_mut(key)
    }
}

impl<K: Eq + Ord, V> Dict<K, V> for BTreeMap<K, V> {
    fn get(&self, key: &K) -> Option<&V> {
        self.get(key)
    }
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.get_mut(key)
    }
}

impl<K: Eq, V> Dict<K, V> for Vec<(K, V)> {
    fn get(&self, key: &K) -> Option<&V> {
        self.iter()
            .find(|(k, _)| k == key)
            .map(|(_, val)| val)
    }
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, val)| val)
    }
}

impl<K: Eq, V, const N: usize> Dict<K, V> for Array<(K, V), N> {
    fn get(&self, key: &K) -> Option<&V> {
        self.0.iter()
            .find(|(k, _)| k == key)
            .map(|(_, val)| val)
    }
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.0.iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, val)| val)
    }
}

impl<K: Eq, V, const N: usize> Dict<K, V> for MaskedArray<(K, V), N> {
    fn get(&self, key: &K) -> Option<&V> {
        self.mask.iter()
            .zip(self.vals.iter())
            .find(|(mask, (k, _))| **mask && k == key)
            .map(|(_, (_, val))| val)
    }
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.mask.iter()
            .zip(self.vals.iter_mut())
            .find(|(mask, (k, _))| **mask && k == key)
            .map(|(_, (_, val))| val)
    }
}


#[repr(transparent)]
pub struct Single<T>(pub T);
impl<T> IntoIterator for Single<T> {
    type Item = T;
    type IntoIter = <Option<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Some(self.0).into_iter()
    }
}


pub struct Array<T, const N: usize>(pub [T; N]);
impl<T, const N: usize> IntoIterator for Array<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.0)
    }
}


pub struct MaskedArray<T, const N: usize> {
    pub mask: [bool; N],
    pub vals: [T; N],
}
impl<T, const N: usize> IntoIterator for MaskedArray<T, N> {
    type Item = T;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.mask)
            .zip(IntoIter::new(self.vals))
            .filter(|(mask, _)| *mask)
            .map(|(_, val)| val)
    }
}