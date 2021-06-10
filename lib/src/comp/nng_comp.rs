// use std::cell::RefCell;
// use std::future::Future;
// use std::convert::TryInto;

// use nng::{Aio, AioResult, Message, Protocol, Socket};

// use crate::collections::Collection;
// use crate::lattice::LatticeRepr;
// use crate::lattice::setunion::{SetUnion};
// use crate::op::OpDelta;

// use super::{Comp, Next};

// pub struct NngComp<O: OpDelta>
// where
//     O::LatRepr: LatticeRepr<Lattice = SetUnion<(String, Message)>>,
//     <O::LatRepr as LatticeRepr>::Repr: Collection<(String, Message), ()>,
// {
//     op: O,
//     _ctxs: Vec<nng::Context>,
//     _aios: Vec<Aio>,
// }

// impl<O: OpDelta> NngComp<O>
// where
//     O::LatRepr: LatticeRepr<Lattice = SetUnion<(String, Message)>>,
//     <O::LatRepr as LatticeRepr>::Repr: Collection<(String, Message), ()>,
// {
//     pub fn new(op: O, url: &str) -> Self {
//         Self {
//             op,
//             tcp_write: RefCell::new(tcp_write),
//         }
//     }
// }

// impl<O: OpDelta, T: Clone + Serialize> Comp for NngComp<O, T>
// where
//     O::LatRepr: LatticeRepr<Lattice = SetUnion<T>>,
//     <O::LatRepr as LatticeRepr>::Repr: Collection<T, ()>,
// {
//     type Error = tokio::io::Error;

//     type TickFuture<'s> = impl Future<Output = Result<(), Self::Error>>;
//     fn tick(&self) -> Self::TickFuture<'_> {
//         async move {
//             if let Some(hide) = (Next { op: &self.op }).await {
//                 for item in hide.reveal_ref().keys() {
//                     let bytes = serde_json::to_vec(item)?;
//                     let mut tcp_write_mut = self.tcp_write.borrow_mut();
//                     let len = bytes.len().try_into().unwrap_or_else(|_| panic!("Message too long! {}", bytes.len()));
//                     tcp_write_mut.write_u16(len).await?;
//                     tcp_write_mut.write_all(&*bytes).await?;
//                 }
//                 Ok(())
//             }
//             else {
//                 Err(tokio::io::Error::new(std::io::ErrorKind::UnexpectedEof, "End of stream."))
//             }
//         }
//     }
// }
