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



pub struct CloneOp<St: RefPullOp>
where
    St::Domain: Clone,
{
    stream: St,
}
impl<St: RefPullOp> CloneOp<St>
where
    St::Domain: Clone,
{
    pub fn new(stream: St) -> Self {
        Self {
            stream: stream,
        }
    }
}
impl<St: RefPullOp> PullOp for CloneOp<St>
where
    St::Domain: Clone,
{
    type Domain = St::Domain;
}
impl<St: RefPullOp> MovePullOp for CloneOp<St>
where
    St::Domain: Clone,
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Domain>> {
        self.stream.poll_next(ctx)
            .map(|opt| opt.map(|x| x.clone()))
    }
}



pub struct LatticeOp<St: MovePullOp, F: Merge<Domain = St::Domain>> {
    stream: St,
    state: F::Domain,
}
impl<St: MovePullOp, F: Merge<Domain = St::Domain>> LatticeOp<St, F> {
    pub fn new(stream: St, bottom: F::Domain) -> Self {
        Self {
            stream: stream,
            state: bottom,
        }
    }
}
impl<St: MovePullOp, F: Merge<Domain = St::Domain>> PullOp for LatticeOp<St, F> {
    type Domain = St::Domain;
}
impl<St: MovePullOp, F: Merge<Domain = St::Domain>> RefPullOp for LatticeOp<St, F> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Domain>> {
        if let Poll::Ready(Some(delta)) = self.stream.poll_next(ctx) {
            F::merge_in(&mut self.state, delta);
        }
        Poll::Ready(Some(&self.state))
    }
}
