use bytes::{BufMut, Bytes, BytesMut};
use tokio::net::{TcpListener, TcpStream};

use spinach::collections::Single;
use spinach::comp::{Comp, TcpComp};
use spinach::func::unary::Morphism;
use spinach::hide::{Hide, Qualifier};
use spinach::lattice::set_union::SetUnionRepr;
use spinach::op::{OnceOp, MergeOp, MorphismOp, TcpOp, BatchConvertOp, DebugOp};
use spinach::tag;


struct Increment;
impl Morphism for Increment {
    type InLatRepr = SetUnionRepr<tag::VEC, BytesMut>;
    type OutLatRepr = SetUnionRepr<tag::VEC, Bytes>;
    fn call<Y: Qualifier>(&self, item: Hide<Y, Self::InLatRepr>) -> Hide<Y, Self::OutLatRepr> {
        item.map(|mut bytes| {
            if let Some(rf) = bytes.get_mut(0) {
                *rf += 1;
            }
            bytes.freeze()
        })
    }
}


#[tokio::test]
pub async fn test_tcp() -> tokio::io::Result<()> {

    let listener = TcpListener::bind("localhost:35575").await?;
    let connector = TcpStream::connect("localhost:35575").await?;
    let (acceptor, _) = listener.accept().await?;
    std::mem::drop(listener);

    let acceptor_op = TcpOp::new(acceptor.into_split().0);
    let convert_op = BatchConvertOp::<_, SetUnionRepr<tag::VEC, BytesMut>>::new(acceptor_op);

    let mut seed = BytesMut::with_capacity(1);
    seed.put_u8(b'a');
    let once_op = OnceOp::<SetUnionRepr<tag::SINGLE, BytesMut>>::new(Single(seed));

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
