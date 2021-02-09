use std::fmt::Debug;
use std::task::{ Context, Poll };

use super::op::*;


pub struct DebugOp<P: Op> {
    tag: &'static str,
    next_pipe: P,
}
impl<P: Op> DebugOp<P> {
    pub fn new(tag: &'static str, next_pipe: P) -> Self {
        DebugOp {
            tag: tag,
            next_pipe: next_pipe,
        }
    }
}
impl<P: Op> Op for DebugOp<P> {
    type Domain = P::Domain;
    type Codomain = P::Codomain;
}
impl<P: MovePullOp> MovePullOp for DebugOp<P>
where
    P::Codomain: Debug,
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Codomain>> {
        let polled = self.next_pipe.poll_next(ctx);
        match &polled {
            Poll::Ready(Some(item)) => println!("{}: {:?}", self.tag, item),
            _ => (),
        }
        polled
    }
}
impl<P: RefPullOp> RefPullOp for DebugOp<P>
where
    P::Codomain: Debug,
{
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Codomain>> {
        let polled = self.next_pipe.poll_next(ctx);
        match &polled {
            Poll::Ready(Some(item)) => println!("{}: {:?}", self.tag, item),
            _ => (),
        }
        polled
    }
}
impl<P: MovePushOp> MovePushOp for DebugOp<P>
where
    P::Domain: Debug,
{
    type Feedback = P::Feedback;

    fn push(&mut self, item: Self::Domain) -> Self::Feedback {
        println!("{}: {:?}", self.tag, item);
        self.next_pipe.push(item)
    }
}
impl<P: RefPushOp> RefPushOp for DebugOp<P>
where
    P::Domain: Debug,
{
    type Feedback = P::Feedback;

    fn push(&mut self, item: &Self::Domain) -> Self::Feedback {
        println!("{}: {:?}", self.tag, item);
        self.next_pipe.push(item)
    }
}
