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

        // Poll from A.
        let eos = match self.op_a.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => {
                let delta = delta.into_reveal();
                for ((k, va), _) in delta.entries() {
                    hashmaps.a.entry(k.clone())
                        .or_default()
                        .insert(va.clone());
                }
                for ((k, va), _) in delta.entries() {
                    if let Some(vbs) = hashmaps.b.get(&k) {
                        out.extend(
                            vbs.iter().cloned()
                                .map(|vb| (k.clone(), va.clone(), vb))
                        );
                    }
                }
                false
            },
            Poll::Ready(None) => true,
            Poll::Pending => false,
        };

        match self.op_b.poll_delta(ctx) {
            Poll::Ready(Some(delta)) => {
                let delta = delta.into_reveal();
                for ((k, vb), _) in delta.entries() {
                    hashmaps.b.entry(k.clone())
                        .or_default()
                        .insert(vb.clone());
                }
                for ((k, vb), _) in delta.entries() {
                    if let Some(vas) = hashmaps.a.get(&k) {
                        out.extend(
                            vas.iter().cloned()
                                .map(|va| (k.clone(), va, vb.clone()))
                        );
                    }
                }
            },
            Poll::Ready(None) => {
                if eos {
                    return Poll::Ready(None);
                }
            },
            Poll::Pending => {},
        };

        if !out.is_empty() {
            Poll::Ready(Some(Hide::new(out)))
        }
        else {
            Poll::Pending
        }
    }
}

pub struct SymHashJoinOrder<A: Order, B: Order>(std::marker::PhantomData<(A, B)>);
impl<A: Order, B: Order> Order for SymHashJoinOrder<A, B> {}
