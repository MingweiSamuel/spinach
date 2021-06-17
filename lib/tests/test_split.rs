use spinach::collections::Array;
use spinach::comp::{CompExt, DebugComp};
use spinach::func::unary::Morphism;
use spinach::hide::{Hide, Qualifier};
use spinach::lattice::setunion::SetUnionRepr;
use spinach::op::{OnceOp, LatticeOp, Splitter, MergeOp, MorphismOp};
use spinach::tag;

struct Mult2Add1;
impl Morphism for Mult2Add1 {
    type InLatRepr = SetUnionRepr<tag::BTREE_SET, usize>;
    type OutLatRepr = SetUnionRepr<tag::BTREE_SET, usize>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        item.map(|x| x * 2 + 1)
    }
}

struct Square;
impl Morphism for Square {
    type InLatRepr = SetUnionRepr<tag::BTREE_SET, usize>;
    type OutLatRepr = SetUnionRepr<tag::BTREE_SET, usize>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        item.map(|x| x * x)
    }
}

#[tokio::test]
pub async fn test_split_merge() -> Result<(), String> {

    type MyLatRepr = SetUnionRepr<tag::ARRAY<10>, usize>;

    let op = OnceOp::<MyLatRepr>::new(Array([
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    ]));
    let op = LatticeOp::<_, SetUnionRepr<tag::BTREE_SET, usize>>::new(op, Default::default());

    let splitter = Splitter::new(op);

    let split0 = splitter.add_split();
    let split0 = MorphismOp::new(split0, Mult2Add1);

    let split1 = splitter.add_split();
    let split1 = MorphismOp::new(split1, Square);

    let merge = MergeOp::new(split0, split1);

    let comp = DebugComp::new(merge, "output");

    comp.run().await.unwrap_err();

    Ok(())
}
