use tokio::net::{TcpListener, TcpStream};

use spinach::collections::Single;
use spinach::comp::TcpComp;
use spinach::func::Morphism;
use spinach::hide::{Hide, Qualifier};
use spinach::lattice::setunion::SetUnionRepr;
use spinach::op::{OnceOp, MergeOp, MorphismOp, TcpOp, BatchConvertOp, DebugOp};
use spinach::tag;


struct Increment;
impl Morphism for Increment {
    type InLatRepr = SetUnionRepr<tag::VEC, usize>;
    type OutLatRepr = SetUnionRepr<tag::VEC, usize>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        item.map(|x| x + 1)
    }
}


#[tokio::test]
pub async fn test_tcp() -> tokio::io::Result<()> {

    let listener = TcpListener::bind("localhost:35575").await?;
    let connector = TcpStream::connect("localhost:35575").await?;
    let (acceptor, _) = listener.accept().await?;
    std::mem::drop(listener);

    let acceptor_op = TcpOp::new(acceptor.into_split().0);
    let convert_op = BatchConvertOp::<_, SetUnionRepr<tag::VEC, usize>>::new(acceptor_op);

    let once_op = OnceOp::<SetUnionRepr<tag::SINGLE, usize>>::new(Single(0));

    let op = MergeOp::new(convert_op, once_op);
    let op = DebugOp::new(op, "test_tcp");
    let op = MorphismOp::new(op, Increment);

    let tcp_comp = TcpComp::new(op, connector.into_split().1);

    tcp_comp.tick().await?;
    tcp_comp.tick().await?;
    tcp_comp.tick().await?;
    tcp_comp.tick().await?;
    tcp_comp.tick().await?;
    tcp_comp.tick().await?;
    tcp_comp.tick().await?;
    tcp_comp.tick().await?;
    tcp_comp.tick().await?;
    tcp_comp.tick().await?;

    Ok(())
}
