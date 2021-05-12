use spinach::collections::Array;
use spinach::comp::DebugComp;
use spinach::lattice::setunion::SetUnionRepr;
use spinach::op::{OnceOp, LatticeOp, Splitter, MergeOp, MorphOp};
use spinach::tag;

#[tokio::test]
pub async fn test_split_merge() -> Result<(), String> {

    type MyLatRepr = SetUnionRepr<tag::ARRAY<10>, usize>;

    let op = OnceOp::<MyLatRepr>::new(Array([
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    ]));
    let op = LatticeOp::<_, SetUnionRepr<tag::BTREE_SET, usize>>::new(op, Default::default());

    let splitter = Splitter::new(op);

    let split0 = splitter.add_split();
    let split0 = MorphOp::<_, SetUnionRepr<tag::BTREE_SET, usize>, _>::new(split0, |batch| batch.map(|x| x * 2 + 1));

    let split1 = splitter.add_split();
    let split1 = MorphOp::<_, SetUnionRepr<tag::BTREE_SET, usize>, _>::new(split1, |batch| batch.map(|x| x * x));

    // let split2 = splitter.add_split();
    // let split3 = splitter.add_split();

    let merge = MergeOp::new(split0, split1);
    // let merge1 = MergeOp::new(split2, split3);
    // let merge = MergeOp::new(merge, merge1);

    let comp = DebugComp::new(merge);

    comp.run().await.unwrap_err();

    // comp.tick().await.unwrap();
    // comp.tick().await.unwrap();
    // comp.tick().await.unwrap();
    // comp.tick().await.unwrap();
    // comp.tick().await.unwrap();
    // comp.tick().await.unwrap();
    // comp.tick().await.unwrap();

    Ok(())
}
