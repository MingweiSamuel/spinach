// use super::*;

// use crate::collections::*;
// use crate::tag;

// pub struct DomPair<LA: Lattice, LB: Lattice> {
//     _phantom: std::marker::PhantomData<(LA, LB)>,
// }
// impl<LA: Lattice, LB: Lattice> Lattice for DomPair<LA, LB> {}

// pub struct DomPairRepr<RA: LatticeRepr, RB: LatticeRepr> {
//     _phantom: std::marker::PhantomData<(RA, RB)>,
// }
// impl<RA: LatticeRepr, RB: LatticeRepr> LatticeRepr for DomPairRepr<RA, RB> {
//     type Lattice = DomPair<RA::Lattice, RB::Lattice>;
//     type Repr = (RA::Repr, RB::Repr);
// }


// impl<K, SelfTag, DeltaTag, SelfLR: LatticeRepr<Lattice = L>, DeltaLR: LatticeRepr<Lattice = L>, L: Lattice> Merge<DomPairRepr<DeltaTag, K, DeltaLR>> for DomPairRepr<SelfTag, K, SelfLR>
// where
//     SelfTag:  MapTag,
//     DeltaTag: MapTag,
//     DomPairRepr<SelfTag,  K, SelfLR>: LatticeRepr<Lattice = MapUnion<K, L>>,
//     DomPairRepr<DeltaTag, K, DeltaLR>: LatticeRepr<Lattice = MapUnion<K, L>>,
//     <DomPairRepr<SelfTag,  K, SelfLR> as LatticeRepr>::Repr: Extend<(K, SelfLR::Repr)> + Collection<K, SelfLR::Repr>,
//     <DomPairRepr<DeltaTag, K, DeltaLR> as LatticeRepr>::Repr: IntoIterator<Item = (K, DeltaLR::Repr)>,
//     SelfLR:  Merge<DeltaLR>,
//     DeltaLR: Convert<SelfLR>,
// {
//     fn merge(this: &mut <DomPairRepr<SelfTag, K, SelfLR> as LatticeRepr>::Repr, delta: <DomPairRepr<DeltaTag, K, DeltaLR> as LatticeRepr>::Repr) {
//         let iter: Vec<(K, SelfLR::Repr)> = delta.into_iter()
//             .filter_map(|(k, v)| {
//                 match this.get_mut(&k) {
//                     Some(target_val) => {
//                         <SelfLR as Merge<DeltaLR>>::merge(target_val, v);
//                         None
//                     }
//                     None => {
//                         let val: SelfLR::Repr = <DeltaLR as Convert<SelfLR>>::convert(v);
//                         Some((k, val))
//                     }
//                 }
//             })
//             .collect();
//         this.extend(iter);
//     }
// }

// fn __assert_merges() {
//     use static_assertions::{assert_impl_all, assert_not_impl_any};
    
//     use super::setunion::{SetUnionRepr};

//     type HashMapHashSet    = DomPairRepr<tag::HASH_MAP, String, SetUnionRepr<tag::HASH_SET, u32>>;
//     type HashMapArraySet   = DomPairRepr<tag::HASH_MAP, String, SetUnionRepr<tag::ARRAY<8>, u32>>;
//     type OptionMapArraySet = DomPairRepr<tag::OPTION,   String, SetUnionRepr<tag::HASH_SET, u32>>;

//     assert_impl_all!(HashMapHashSet: Merge<HashMapHashSet>);
//     assert_impl_all!(HashMapHashSet: Merge<HashMapArraySet>);

//     assert_not_impl_any!(HashMapArraySet: Merge<HashMapHashSet>);
//     assert_not_impl_any!(HashMapArraySet: Merge<HashMapArraySet>);

//     assert_not_impl_any!(OptionMapArraySet: Merge<HashMapHashSet>);
//     assert_not_impl_any!(OptionMapArraySet: Merge<HashMapArraySet>);
// }
