use std::cmp::Ordering;
use std::collections::btree_map;
use std::collections::hash_map;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use super::Lattice;

// MAP MERGES //

/// Map union composed lattice.
pub struct MapUnion<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<K, F> Lattice for MapUnion<HashMap<K, F>>
where
    K: Hash + Eq,
    F: Lattice,
{
    type Domain = HashMap<K, <F as Lattice>::Domain>;

    fn merge_in(val: &mut Self::Domain, delta: Self::Domain) {
        for (k, v) in delta {
            match val.entry(k) {
                hash_map::Entry::Occupied(mut kv) => {
                    F::merge_in(kv.get_mut(), v);
                }
                hash_map::Entry::Vacant(kv) => {
                    kv.insert(v);
                }
            }
        }
    }

    // TODO: these are awful looking, and also need testing. Could use helper method.
    fn partial_cmp(val: &Self::Domain, delta: &Self::Domain) -> Option<Ordering> {
        // Ordering::Equal OR Ordering::Greater
        if val.len() >= delta.len() {
            let mut result = None;
            for (k, delta_val) in delta {
                match val.get(k) {
                    Some(val_val) => {
                        let cmp = F::partial_cmp(val_val, delta_val);
                        match cmp {
                            Some(cmp) => {
                                if result.get_or_insert(cmp) != &cmp {
                                    return None;
                                }
                            }
                            None => return None,
                        }
                    }
                    None => return None,
                }
            }
            if None == result {
                Some(Ordering::Equal)
            } else {
                Some(Ordering::Greater)
            }
        }
        // Ordering::Less
        else {
            for (k, val_val) in val {
                match delta.get(k) {
                    Some(delta_val) => {
                        let cmp = F::partial_cmp(val_val, delta_val);
                        if Some(Ordering::Less) != cmp {
                            return None;
                        }
                    }
                    None => return None,
                }
            }
            Some(Ordering::Less)
        }
    }

    fn delta(val: &Self::Domain, delta: &mut Self::Domain) -> bool {
        delta.retain(|delta_key, delta_val| {
            match val.get(&delta_key) {
                Some(val_val) => F::delta(val_val, delta_val),
                None => true,
            }
        });
        !delta.is_empty()
    }

    // fn remainder(val: &mut Self::Domain, delta: Self::Domain) -> bool {
    //     for (k, v) in delta {
    //         match val.entry(k) {
    //             hash_map::Entry::Occupied(mut kv) => {
    //                 if F::remainder(kv.get_mut(), v) {
    //                     // If value is dominated, remove it.
    //                     kv.remove_entry();
    //                 }
    //             }
    //             hash_map::Entry::Vacant(kv) => {
    //                 kv.insert(v);
    //             }
    //         }
    //     }
    //     val.is_empty()
    // }
}

impl<K, F> Lattice for MapUnion<BTreeMap<K, F>>
where
    K: Ord + Eq,
    F: Lattice,
{
    type Domain = BTreeMap<K, <F as Lattice>::Domain>;

    fn merge_in(val: &mut Self::Domain, delta: Self::Domain) {
        for (k, v) in delta {
            match val.entry(k) {
                btree_map::Entry::Occupied(mut kv) => {
                    F::merge_in(kv.get_mut(), v);
                }
                btree_map::Entry::Vacant(kv) => {
                    kv.insert(v);
                }
            }
        }
    }

    // TODO: these are awful looking, and also need testing. Could use helper method.
    fn partial_cmp(val: &Self::Domain, delta: &Self::Domain) -> Option<Ordering> {
        // Ordering::Equal OR Ordering::Greater
        if val.len() >= delta.len() {
            let mut result = None;
            for (k, delta_val) in delta {
                match val.get(k) {
                    Some(val_val) => {
                        let cmp = F::partial_cmp(val_val, delta_val);
                        match cmp {
                            Some(cmp) => {
                                if result.get_or_insert(cmp) != &cmp {
                                    return None;
                                }
                            }
                            None => return None,
                        }
                    }
                    None => return None,
                }
            }
            if None == result {
                Some(Ordering::Equal)
            } else {
                Some(Ordering::Greater)
            }
        }
        // Ordering::Less
        else {
            for (k, val_val) in val {
                match delta.get(k) {
                    Some(delta_val) => {
                        let cmp = F::partial_cmp(val_val, delta_val);
                        if Some(Ordering::Less) != cmp {
                            return None;
                        }
                    }
                    None => return None,
                }
            }
            Some(Ordering::Less)
        }
    }

    fn delta(val: &Self::Domain, delta: &mut Self::Domain) -> bool {
        delta.retain(|delta_key, delta_val| {
            match val.get(&delta_key) {
                Some(val_val) => F::delta(val_val, delta_val),
                None => true,
            }
        });
        !delta.is_empty()
    }

    // fn remainder(val: &mut Self::Domain, delta: Self::Domain) -> bool {
    //     for (k, v) in delta {
    //         match val.entry(k) {
    //             btree_map::Entry::Occupied(mut kv) => {
    //                 if F::remainder(kv.get_mut(), v) {
    //                     // If value is dominated, remove it.
    //                     kv.remove_entry();
    //                 }
    //             }
    //             btree_map::Entry::Vacant(kv) => {
    //                 kv.insert(v);
    //             }
    //         }
    //     }
    //     val.is_empty()
    // }
}

// pub struct MapIntersection<T> {
//     _phantom: std::marker::PhantomData<T>,
// }
// impl <K, F> Lattice for MapIntersection<HashMap<K, F>>
// where
//     K: Eq + Hash,
//     F: Lattice,
// {
//     type Domain = HashMap<K, <F as Lattice>::Domain>;

//     fn merge_in(val: &mut Self::Domain, delta: Self::Domain) {
//         todo!("this is broken.");
//         for (k, v) in delta {
//             val.entry(k).and_modify(|v0| F::merge_in(v0, v));
//         }
//     }

//     fn partial_cmp(val: &Self::Domain, delta: &Self::Domain) -> Option<Ordering> {
//         todo!("this is broken.");
//         // Ordering::Equal OR Ordering::Less
//         if val.len() >= delta.len() {
//             let mut result = None;
//             for (k, delta_val) in delta {
//                 match val.get(k) {
//                     Some(val_val) => {
//                         let cmp = F::partial_cmp(&val_val, delta_val);
//                         match cmp {
//                             Some(cmp) => {
//                                 if result.get_or_insert(cmp) != &cmp {
//                                     return None;
//                                 }
//                             },
//                             None => return None,
//                         }
//                     },
//                     None => return None,
//                 }
//             }
//             if None == result {
//                 return Some(Ordering::Equal);
//             }
//             else {
//                 return Some(Ordering::Less);
//             }
//         }
//         // Ordering::Greater
//         else {
//             for (k, val_val) in val {
//                 match delta.get(k) {
//                     Some(delta_val) => {
//                         let cmp = F::partial_cmp(&val_val, delta_val);
//                         if Some(Ordering::Greater) != cmp {
//                             return None;
//                         }
//                     },
//                     None => return None,
//                 }
//             }
//             return Some(Ordering::Greater);
//         }
//     }
// }
