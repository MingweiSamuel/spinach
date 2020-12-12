use std::task::{ Context, Poll };

use crate::merge::Merge;


pub trait PullOp {
    type Domain;
}
pub trait MovePullOp: PullOp {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Domain>>;
}
pub trait RefPullOp: PullOp {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Domain>>;
}



pub struct CloneOp<O: RefPullOp>
where
    O::Domain: Clone,
{
    op: O,
}
impl<O: RefPullOp> CloneOp<O>
where
    O::Domain: Clone,
{
    pub fn new(op: O) -> Self {
        Self {
            op: op,
        }
    }
}
impl<O: RefPullOp> PullOp for CloneOp<O>
where
    O::Domain: Clone,
{
    type Domain = O::Domain;
}
impl<St: RefPullOp> MovePullOp for CloneOp<St>
where
    St::Domain: Clone,
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Domain>> {
        self.op.poll_next(ctx)
            .map(|opt| opt.map(|x| x.clone()))
    }
}



pub struct LatticeOp<O: MovePullOp, F: Merge<Domain = O::Domain>> {
    op: O,
    state: F::Domain,
}
impl<O: MovePullOp, F: Merge<Domain = O::Domain>> LatticeOp<O, F> {
    pub fn new(op: O, bottom: F::Domain) -> Self {
        Self {
            op: op,
            state: bottom,
        }
    }
}
impl<O: MovePullOp, F: Merge<Domain = O::Domain>> PullOp for LatticeOp<O, F> {
    type Domain = O::Domain;
}
impl<St: MovePullOp, F: Merge<Domain = St::Domain>> RefPullOp for LatticeOp<St, F> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Domain>> {
        if let Poll::Ready(Some(delta)) = self.op.poll_next(ctx) {
            F::merge_in(&mut self.state, delta);
        }
        Poll::Ready(Some(&self.state))
    }
}
