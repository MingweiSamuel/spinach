use std::hash::Hash;
use std::pin::Pin;
use std::future::Future;
use std::task::{ Context, Poll };

use spinach::tag;

use spinach::hide::{ Hide, Delta, Value };
// use spinach::op::Op;
use spinach::lattice::{ LatticeRepr, Merge };
// use spinach::lattice::pair::PairRepr;
use spinach::lattice::set_union::SetUnionRepr;
use spinach::lattice::map_union::MapUnionRepr;

use futures::future::{ select_all, select, Either };

pub trait In {
    type LatRepr: LatticeRepr;

    fn push_delta(&self, item: Hide<Delta, Self::LatRepr>);
}

pub trait Out {
    type LatRepr: LatticeRepr;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>>;
}


pub type Graph<I, O> = MapUnionRepr<tag::VEC, O, SetUnionRepr<tag::VEC, I>>;

// pub type MySpecificGraph = Graph<dyn In, dyn Out<LatRepr = I::LatRepr>>

pub async fn run<I, O, U>(mut graph: Hide<Value, Graph<I, O>>, updates: U)
where
    I: In + Eq + Hash + Clone + 'static,
    O: Out<LatRepr = I::LatRepr> + Eq + Hash + Clone + 'static,
    U: Out<LatRepr = Graph<I, O>> + 'static,
{
    loop {
        let update = {
            // Future to update the graph.
            let update_fut = OutFuture(&updates);
            // Future to push things around.
            let tick_fut = Box::pin(async {
                let graph = graph.reveal_ref(); // !!!!
                // TODO: Use streams to not rebuild every time.
                let futures = graph.iter().map(|(o, _i)| o).map(OutFuture);
                let (item, idx, _others) = select_all(futures).await;
                for out in &graph[idx].1 {
                    out.push_delta(item.clone());
                }
            });

            // Run either.
            match select(update_fut, tick_fut).await {
                Either::Left((update, _)) => Some(update),
                Either::Right(_) => None,
            }
        };

        if let Some(update) = update {
            Merge::merge_hide(&mut graph, update);
        }
    }
}

pub fn main() {
    // let graph = dyn
}


pub struct OutFuture<'a, O: Out>(&'a O);
impl<'a, O: Out> Future for OutFuture<'a, O> {
    type Output = Hide<Delta, O::LatRepr>;
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0.poll_delta(ctx) {
            Poll::Ready(Some(item)) => Poll::Ready(item),
            Poll::Ready(None) => panic!("EOS"),
            Poll::Pending => Poll::Pending,
        }
    }
}

// #[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
// pub struct SplitId(&'static str);

// pub type InsOuts<I: In + Eq, O: Out + Eq> = PairRepr<
//     SetUnionRepr<tag::HASH_SET, I>,
//     SetUnionRepr<tag::HASH_SET, O>>;
// pub type SplitTable<I: In + Eq, O: Out + Eq> = MapUnionRepr<tag::HASH_MAP, SplitId, InsOuts<I, O>>;


// pub type PredTable<O: Out + Eq, I: In + Eq> = SetUnionRepr<tag::VEC, (O, I)>;
// pub struct DispatchRow {
//     key: &'static str,
//     pred:
// }
// type DispatchTable = MapUnionRepr<tag::HASH_MAP,
