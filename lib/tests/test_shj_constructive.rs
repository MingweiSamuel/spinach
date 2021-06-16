use spinach::collections::Single;
use spinach::comp::{CompExt, DebugComp};
use spinach::func::binary::{CartesianProduct, HashPartitioned};
use spinach::lattice::mapunion::MapUnionRepr;
use spinach::lattice::setunion::SetUnionRepr;
use spinach::op::{BinaryOp, IterOp, LatticeOp};
use spinach::tag;

#[tokio::test]
pub async fn test_shj_constructive() -> Result<(), String> {

    type InputLatRepr = MapUnionRepr<tag::SINGLE, &'static str, SetUnionRepr<tag::SINGLE, &'static str>>;
    type LatticeLatRepr = MapUnionRepr<tag::HASH_MAP, &'static str, SetUnionRepr<tag::HASH_SET, &'static str>>;
    type JoinedInnerLatRepr = SetUnionRepr<tag::HASH_SET, (&'static str, &'static str)>;

    let op_a = IterOp::<InputLatRepr, _>::new(vec![
        Single(("P", Single("Pranav"))),
        Single(("M", Single("Matthew"))),
        Single(("M", Single("Mingwei"))),
        Single(("J", Single("Joseph"))),
    ]);
    let op_a = LatticeOp::<_, LatticeLatRepr>::new_default(op_a);

    let op_b = IterOp::<InputLatRepr, _>::new(vec![
        Single(("M", Single("May"))),
        Single(("M", Single("March"))),
        Single(("J", Single("June"))),
        Single(("J", Single("July"))),
        Single(("D", Single("December"))),
    ]);
    let op_b = LatticeOp::<_, LatticeLatRepr>::new_default(op_b);

    let binary_func = HashPartitioned::new(CartesianProduct::<_, _, _, _, JoinedInnerLatRepr>::new());
    let op = BinaryOp::new(op_a, op_b, binary_func);

    // let op = SymHashJoinOp::new(op_a, op_b);
    let comp = DebugComp::new(op, "output");

    comp.run().await.unwrap_err();

    Ok(())
}
