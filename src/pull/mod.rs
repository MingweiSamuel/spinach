use std::cell::{ Cell, RefCell };
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{ Context, Poll, Waker };

use tokio::stream::Stream;
use tokio::sync::mpsc;

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



pub struct MoveNext<'a, O: MovePullOp> {
    op: &'a mut O,
}
impl<'a, O: MovePullOp> MoveNext<'a, O> {
    pub fn new(op: &'a mut O) -> Self {
        Self {
            op: op,
        }
    }
}
impl<O: MovePullOp> Future for MoveNext<'_, O>
where
    Self: Unpin,
{
    type Output = Option<O::Domain>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_mut().op.poll_next(ctx)
    }
}



pub struct NoOp<T> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T> NoOp<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<T> PullOp for NoOp<T> {
    type Domain = T;
}
impl<T> MovePullOp for NoOp<T> {
    fn poll_next(&mut self, _ctx: &mut Context<'_>) -> Poll<Option<Self::Domain>> {
        Poll::Pending
    }
}
impl<T> RefPullOp for NoOp<T> {
    fn poll_next(&mut self, _ctx: &mut Context<'_>) -> Poll<Option<&Self::Domain>> {
        Poll::Pending
    }
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



pub struct MapFilterOp<O: PullOp, F, T> {
    op: O,
    func: F,
    _phantom: std::marker::PhantomData<T>,
}
impl<O: PullOp, F, T> MapFilterOp<O, F, T> {
    pub fn new(op: O, func: F) -> Self {
        Self {
            op: op,
            func: func,
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<O: PullOp, F, T> PullOp for MapFilterOp<O, F, T> {
    type Domain = T;
}
impl<O: MovePullOp, F: Fn(O::Domain) -> Option<T>, T> MovePullOp for MapFilterOp<O, F, T> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Domain>> {
        let val = self.op.poll_next(ctx);
        val.map(|opt| opt.and_then(|x| (self.func)(x)))
    }
}



// pub struct RefMapFilterOp<O: PullOp, F, T> {
//     op: O,
//     func: Option<F>,
//     _phantom: std::marker::PhantomData<T>,
// }
// impl<O: PullOp, F, T> RefMapFilterOp<O, F, T> {
//     pub fn new(op: O, func: F) -> Self {
//         Self {
//             op: op,
//             func: Some(func),
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }
// impl<O: PullOp, F, T> PullOp for RefMapFilterOp<O, F, T> {
//     type Domain = T;
// }
// impl<O: RefPullOp, F: Fn(&O::Domain) -> Option<T>, T> MovePullOp for RefMapFilterOp<O, F, T> {
//     fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Domain>> {
//         // Hack for partial ownership/downgrading ref.
//         let func = self.func.take().unwrap();
//         let val = self.op.poll_next(ctx)
//             .map(|opt| opt.and_then(|x| (func)(x)));
//         self.func.replace(func);
//         val
//     }
// }



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
impl<O: MovePullOp, F: Merge<Domain = O::Domain>> RefPullOp for LatticeOp<O, F> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Domain>> {
        if let Poll::Ready(opt) = self.op.poll_next(ctx) {
            match opt {
                Some(delta) => F::merge_in(&mut self.state, delta),
                None => return Poll::Ready(None), // EoS.
            }
        }
        Poll::Ready(Some(&self.state))
    }
}




pub struct LatticeOp2<O: MovePullOp, F: Merge<Domain = O::Domain>> {
    op: Rc<RefCell<O>>,
    state: Rc<RefCell<F::Domain>>,
    id_counter: Rc<Cell<usize>>,
    id: usize,
    wakers: Rc<RefCell<HashMap<usize, Waker>>>,
}
impl<O: MovePullOp, F: Merge<Domain = O::Domain>> LatticeOp2<O, F> {
    pub fn new(op: O, bottom: F::Domain) -> Self {
        Self {
            op: Rc::new(RefCell::new(op)),
            state: Rc::new(RefCell::new(bottom)),
            id_counter: Rc::new(Cell::new(0)),
            id: 0,
            wakers: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}
impl<O: MovePullOp, F: Merge<Domain = O::Domain>> PullOp for LatticeOp2<O, F> {
    type Domain = Rc<RefCell<O::Domain>>;
}
impl<O: MovePullOp, F: Merge<Domain = O::Domain>> MovePullOp for LatticeOp2<O, F> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Domain>> {

        if let Poll::Ready(Some(delta)) = self.op.borrow_mut().poll_next(ctx) {
            F::merge_in(&mut *self.state.borrow_mut(), delta);

            // New data. Wake everyone.
            let wakers_borrow = self.wakers.borrow();
            for ( id, waker ) in &*wakers_borrow {
                if self.id != *id {
                    waker.wake_by_ref();
                }
            }
        }

        self.wakers.borrow_mut().insert(self.id, ctx.waker().clone());

        // Note: even if upstream is closed, this remains open.
        Poll::Ready(Some(self.state.clone()))
    }
}
impl<O: MovePullOp, F: Merge<Domain = O::Domain>> Clone for LatticeOp2<O, F> {
    fn clone(&self) -> Self {
        let id = self.id_counter.update(|x| x + 1);
        Self {
            op: self.op.clone(),
            state: self.state.clone(),
            id_counter: self.id_counter.clone(),
            id: id,
            wakers: self.wakers.clone(),
        }
    }
}



pub struct ChannelOp<T> {
    receiver: mpsc::Receiver<T>,
}
impl<T> ChannelOp<T> {
    pub fn new(receiver: mpsc::Receiver<T>) -> Self {
        Self {
            receiver: receiver,
        }
    }
}
impl<T> PullOp for ChannelOp<T> {
    type Domain = T;
}
impl<T> MovePullOp for ChannelOp<T> {
    fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<Self::Domain>> {
        Pin::new(&mut self.receiver).poll_next(ctx)
    }
}



// pub struct SplitOp<O: RefPullOp> {
//     op: Rc<RefCell<O>>,
// }
// impl<O: RefPullOp> SplitOp<O> {
//     pub fn new(op: O) -> Self {
//         Self {
//             op: Rc::new(RefCell::new(op)),
//         }
//     }
// }
// impl<O: RefPullOp> PullOp for SplitOp<O> {
//     type Domain = O::Domain;
// }
// impl<O: RefPullOp> RefPullOp for SplitOp<O> {
//     fn poll_next(&mut self, ctx: &mut Context<'_>) -> Poll<Option<&Self::Domain>> {
//         let mut borrow = self.op.borrow_mut();
//         let mut borrow = RefMut::map(borrow, |op| {
//             op.poll_next(ctx)
//         });

//         Poll::Pending
//     }
// }




// pub struct Splitter<O: RefPullOp, P: MovePullOp>
// where
//     P::Domain: RefPullOp<Domain = O::Domain>,
// {
//     op: O,
//     pipes_op: P,
//     pipes: Vec<P::Domain>,
// }
// impl<O: RefPullOp, P: MovePullOp> Splitter<O, P>
// where
//     P::Domain: RefPullOp<Domain = O::Domain>,
// {
//     pub fn new(op: O, pipes_op: P) -> Self {
//         Self {
//             op: op,
//             pipes_op: pipes_op,
//             pipes: Vec::new(),
//         }
//     }
// }
// impl<O: RefPullOp, P: MovePullOp> Future for Splitter<O, P>
// where
//     P::Domain: RefPullOp<Domain = O::Domain>,
//     Self: Unpin,
// {
//     type Output = ();

//     fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
//         let me = self.get_mut();
//         while let Poll::Ready(Some(pipe)) = me.pipes_op.poll_next(ctx) {
//             me.pipes.push(pipe);
//         }
//         if let Poll::Ready(opt) = me.op.poll_next(ctx) {
//             match opt {
//                 Some(val) => {
//                     for pipe in me.pipes {
//                         pipe.
//                     }
//                 },
//                 None => return Poll::Ready(()), // EoS
//             }
//         }
//         Poll::Pending
//     }
// }
