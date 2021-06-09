use spinach::collections::Single;
use spinach::comp::{CompExt, DebugComp};
use spinach::lattice::setunion::SetUnionRepr;
use spinach::op::{IterOp, SymHashJoinOp};
use spinach::tag;

#[tokio::test]
pub async fn test_sym_hash_join() -> Result<(), String> {

    type MyLatRepr = SetUnionRepr<tag::SINGLE, (&'static str, &'static str)>;

    let op_a = IterOp::<MyLatRepr, _>::new(vec![
        Single(("P", "Pranav")),
        Single(("M", "Matthew")),
        Single(("M", "Mingwei")),
        Single(("J", "Joseph")),
    ]);

    let op_b = IterOp::<MyLatRepr, _>::new(vec![
        Single(("M", "May")),
        Single(("M", "March")),
        Single(("J", "June")),
        Single(("J", "July")),
        Single(("D", "December")),
    ]);

    let op = SymHashJoinOp::new(op_a, op_b);
    let comp = DebugComp::new(op);

    comp.run().await.unwrap_err();

    Ok(())
}
