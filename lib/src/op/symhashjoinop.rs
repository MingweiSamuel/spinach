use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::hash::Hash;
use std::task::{Context, Poll};

use crate::collections::Collection;
use crate::hide::{Hide, Delta};
use crate::lattice::{LatticeRepr};
use crate::lattice::setunion::{SetUnion, SetUnionRepr};
use crate::metadata::Order;
use crate::tag;

use super::*;

struct HashMaps<K, VA, VB> {
    a: HashMap<K, HashSet<VA>>,
    b: HashMap<K, HashSet<VB>>,
}

pub struct SymHashJoinOp<A: OpDelta, B: OpDelta, K, VA, VB>
where
    K: Eq + Hash + Clone,
    VA: Eq + Hash + Clone,
    VB: Eq + Hash + Clone,
    A::LatRepr: LatticeRepr<Lattice = SetUnion<(K, VA)>>,
    B::LatRepr: LatticeRepr<Lattice = SetUnion<(K, VB)>>,
    <A::LatRepr as LatticeRepr>::Repr: Collection<(K, VA), ()>,
    <B::LatRepr as LatticeRepr>::Repr: Collection<(K, VB), ()>,
{
    op_a: A,
    op_b: B,
    hashmaps: RefCell<HashMaps<K, VA, VB>>,
}

impl<A: OpDelta, B: OpDelta, K, VA, VB> SymHashJoinOp<A, B, K, VA, VB>
where
    K: Eq + Hash + Clone,
    VA: Eq + Hash + Clone,
    VB: Eq + Hash + Clone,
    A::LatRepr: LatticeRepr<Lattice = SetUnion<(K, VA)>>,
    B::LatRepr: LatticeRepr<Lattice = SetUnion<(K, VB)>>,
    <A::LatRepr as LatticeRepr>::Repr: Collection<(K, VA), ()>,
    <B::LatRepr as LatticeRepr>::Repr: Collection<(K, VB), ()>,
{
    pub fn new(op_a: A, op_b: B) -> Self {
        Self {
            op_a,
            op_b,
            hashmaps: RefCell::new(HashMaps {
                a: Default::default(),
                b: Default::default(),
            }),
        }
    }
}

impl<A: OpDelta, B: OpDelta, K, VA, VB> Op for SymHashJoinOp<A, B, K, VA, VB>
where
    K: Eq + Hash + Clone,
    VA: Eq + Hash + Clone,
    VB: Eq + Hash + Clone,
    A::LatRepr: LatticeRepr<Lattice = SetUnion<(K, VA)>>,
    B::LatRepr: LatticeRepr<Lattice = SetUnion<(K, VB)>>,
    <A::LatRepr as LatticeRepr>::Repr: Collection<(K, VA), ()>,
    <B::LatRepr as LatticeRepr>::Repr: Collection<(K, VB), ()>,
{
    type LatRepr = SetUnionRepr<tag::VEC, (K, VA, VB)>;
}

impl<A: OpDelta, B: OpDelta, K, VA, VB> OpDelta for SymHashJoinOp<A, B, K, VA, VB>
where
    K: Eq + Hash + Clone,
    VA: Eq + Hash + Clone,
    VB: Eq + Hash + Clone,
    A::LatRepr: LatticeRepr<Lattice = SetUnion<(K, VA)>>,
    B::LatRepr: LatticeRepr<Lattice = SetUnion<(K, VB)>>,
    <A::LatRepr as LatticeRepr>::Repr: Collection<(K, VA), ()>,
    <B::LatRepr as LatticeRepr>::Repr: Collection<(K, VB), ()>,
{
    type Ord = SymHashJoinOrder<A::Ord, B::Ord>;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        let mut out = Vec::new();
        let mut hashmaps = self.hashmaps.borrow_mut();

        while out.is_empty() {
            // Poll both A and B.

            // (Poll<Option<Hide<Delta, A::LatRepr>>>, Poll<Option<Hide<Delta, B::LatRepr>>>)
            let polls = (self.op_a.poll_delta(ctx), self.op_b.poll_delta(ctx));

            // If both streams are EOS, we are EOS.
            if let (Poll::Ready(None), Poll::Ready(None)) = polls {
                return Poll::Ready(None)
            }
            // If both streams are pending, we are pending.
            if let (Poll::Pending, Poll::Pending) = polls {
                return Poll::Pending
            }

            // Handle new A values.
            if let Poll::Ready(Some(delta_a)) = polls.0 {
                // Build-into A hashmap.
                let delta_a = delta_a.into_reveal();
                for ((k, va), _) in delta_a.entries() {
                    hashmaps.a.entry(k.clone())
                        .or_default()
                        .insert(va.clone());
                }
                // Probe B hashmap.
                for ((k, va), _) in delta_a.entries() {
                    if let Some(vbs) = hashmaps.b.get(&k) {
                        out.extend(
                            vbs.iter().cloned()
                                .map(|vb| (k.clone(), va.clone(), vb))
                        );
                    }
                }
            }

            // Handle new B values.
            if let Poll::Ready(Some(delta_b)) = polls.1 {
                // Build-into B hashmap.
                let delta_b = delta_b.into_reveal();
                for ((k, vb), _) in delta_b.entries() {
                    hashmaps.b.entry(k.clone())
                        .or_default()
                        .insert(vb.clone());
                }
                // Probe A hashmap.
                for ((k, vb), _) in delta_b.entries() {
                    if let Some(vas) = hashmaps.a.get(&k) {
                        out.extend(
                            vas.iter().cloned()
                                .map(|va| (k.clone(), va, vb.clone()))
                        );
                    }
                }
            }
        }

        // Return new tuples in the vec.
        Poll::Ready(Some(Hide::new(out)))
    }
}

pub struct SymHashJoinOrder<A: Order, B: Order>(std::marker::PhantomData<(A, B)>);
impl<A: Order, B: Order> Order for SymHashJoinOrder<A, B> {}
