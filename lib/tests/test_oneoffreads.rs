use tokio::sync::mpsc;

use spinach::comp::*;
use spinach::lattice::ord::{MaxRepr};
use spinach::hide::Hide;
use spinach::op::*;

#[tokio::test]
pub async fn test_oneoffreads() -> Result<(), String> {
    type MyLatticeRepr = MaxRepr<u64>;

    let local = tokio::task::LocalSet::new();
    local.run_until(async {
        let (send_write, recv_write) = mpsc::unbounded_channel();

        let splitter = ChannelOp::<MyLatticeRepr>::new(recv_write)
            .lattice_default()
            .dyn_split();

        let updater = splitter.add_split().comp_debug("Updater");
        let _updater_task = tokio::task::spawn_local(async move {
            println!("{:?}", updater.run().await.unwrap_err());
        });

        send_write.send(Hide::new(1)).map_err(|e| e.to_string())?;
        tokio::task::yield_now().await;
        send_write.send(Hide::new(3)).map_err(|e| e.to_string())?;
        send_write.send(Hide::new(2)).map_err(|e| e.to_string())?;

        let read_a = splitter.add_split()
            .topbox()
            .comp_debug("Read A");
        println!("Read A Done: {:?}", read_a.run().await.unwrap_err());

        send_write.send(Hide::new(4)).map_err(|e| e.to_string())?;
        tokio::task::yield_now().await;
        send_write.send(Hide::new(5)).map_err(|e| e.to_string())?;

        let read_b = splitter.add_split()
            .topbox()
            .comp_debug("Read B");
        println!("Read B Done: {:?}", read_b.run().await.unwrap_err());

        tokio::task::yield_now().await;
        let _ = _updater_task;

        Ok(())
    }).await
}
