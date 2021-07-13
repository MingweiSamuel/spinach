use std::rc::Rc;
use std::pin::Pin;
use std::future::Future;
use std::task::{ Context, Poll };

use spinach::tag;

use spinach::tokio;
use spinach::collections::Array;
use spinach::hide::{ Hide, Delta, Value };
use spinach::op::{ OpDelta, OnceOp };
use spinach::lattice::{ LatticeRepr, Merge };
// use spinach::lattice::pair::PairRepr;
use spinach::lattice::set_union::SetUnionRepr;
use spinach::lattice::map_union::MapUnionRepr;

use futures::future::{ select_all, select, Either };

pub trait In {
    type LatRepr: LatticeRepr;

    fn push_delta(&self, item: Hide<Delta, Self::LatRepr>);
}

impl<I: In> In for Rc<I> {
    type LatRepr = I::LatRepr;

    fn push_delta(&self, item: Hide<Delta, Self::LatRepr>) {
        I::push_delta(self, item);
    }
}

pub trait Out {
    type LatRepr: LatticeRepr;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>>;
}

impl<O: OpDelta> Out for Rc<O> {
    type LatRepr = O::LatRepr;

    fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
        O::poll_delta(self, ctx)
    }
}

// impl<O: Out> Out for Rc<O> {
//     type LatRepr = O::LatRepr;

//     fn poll_delta(&self, ctx: &mut Context<'_>) -> Poll<Option<Hide<Delta, Self::LatRepr>>> {
//         self.poll_delta(ctx)
//     }
// }

pub type Graph<O, I> = MapUnionRepr<tag::VEC, O, SetUnionRepr<tag::VEC, I>>;

// pub type MySpecificGraph = Graph<dyn In, dyn Out<LatRepr = I::LatRepr>>

pub async fn run<I, O, U>(mut graph: Hide<Value, Graph<O, I>>, updates: U)
where
    I: In + Eq + Clone + 'static,
    O: Out<LatRepr = I::LatRepr> + Eq + Clone + 'static,
    U: Out<LatRepr = Graph<O, I>> + 'static,
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

#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        type MyLatRepr = SetUnionRepr<tag::ARRAY<10>, usize>;

        #[derive(PartialEq, Eq)]
        struct DebugSink();
        impl In for DebugSink {
            type LatRepr = MyLatRepr;
            fn push_delta(&self, item: Hide<Delta, Self::LatRepr>) {
                for x in item.into_reveal() {
                    println!("{}", x);
                }
            }
        }


        let op_a = Rc::new(OnceOp::<MyLatRepr>::new(Array([
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
        ])));

        let op_b = Rc::new(OnceOp::<MyLatRepr>::new(Array([
            5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
        ])));

        type MyGraph = Graph<Rc<OnceOp<MyLatRepr>>, Rc<DebugSink>>;

        let sink = Rc::new(DebugSink());

        let graph_a: Hide<Value, MyGraph> = Hide::new(vec![
            (op_a, vec![ sink.clone() ])
        ]);

        let graph_b: <MyGraph as LatticeRepr>::Repr = vec![
            (op_b, vec![ sink.clone() ])
        ];
        let graph_updates = Rc::new(OnceOp::<MyGraph>::new(graph_b));

        tokio::task::spawn_local(async move {
            run(graph_a, graph_updates).await
        }).await.unwrap();

    }).await;
}


pub struct OutFuture<'a, O: Out>(&'a O);
impl<'a, O: Out> Future for OutFuture<'a, O> {
    type Output = Hide<Delta, O::LatRepr>;
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0.poll_delta(ctx) {
            Poll::Ready(Some(item)) => Poll::Ready(item),
            Poll::Ready(None) => Poll::Pending,
            Poll::Pending => Poll::Pending,
        }
    }
}
