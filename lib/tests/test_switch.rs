use spinach::collections::Array;
use spinach::comp::{CompExt, NullComp};
use spinach::func::unary::Morphism;
use spinach::hide::{Hide, Qualifier};
use spinach::lattice::setunion::SetUnionRepr;
use spinach::lattice::pair::PairRepr;
use spinach::op::{DebugOp, LatticeOp, MergeOp, MorphismOp, OnceOp, SwitchOp};
use spinach::tag;


struct EvenOdd;
impl Morphism for EvenOdd {
    type InLatRepr = SetUnionRepr<tag::BTREE_SET, usize>;
    type OutLatRepr = PairRepr<SetUnionRepr<tag::VEC, usize>, SetUnionRepr<tag::VEC, usize>>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        // TODO: cloning misses the point of switching.
        Hide::zip(
            item.clone().filter(|x| 0 == x % 2),
            item        .filter(|x| 1 == x % 2))
    }
}

#[tokio::test]
pub async fn test_switch() -> Result<(), String> {

    type MyLatRepr = SetUnionRepr<tag::ARRAY<10>, usize>;

    let op = OnceOp::<MyLatRepr>::new(Array([
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    ]));
    let op = LatticeOp::<_, SetUnionRepr<tag::BTREE_SET, usize>>::new(op, Default::default());

    let op = MorphismOp::new(op, EvenOdd);

    let (op_a, op_b) = SwitchOp::new(op);
    let comp_a = DebugOp::new(op_a, "A");
    let comp_b = DebugOp::new(op_b, "B");

    let merge = MergeOp::new(comp_a, comp_b);
    let comp = NullComp::new(merge);

    comp.run().await.unwrap_err();

    Ok(())
}
