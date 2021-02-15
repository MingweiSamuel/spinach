use spinach::op2::{ MovePushOp, DebugOp, MapFilterMoveOp, channel_op, handoff_op, DF, StaticComp };

#[tokio::test]
pub async fn test_cycle_channel() -> Result<(), String> {

    let ( push_pipe, pull_pipe ) = channel_op::<usize>(10);
    let mut push_pipe = push_pipe;

    let pull_pipe = MapFilterMoveOp::<_, _, usize>::new(pull_pipe, |x: usize| Some(x + 1));
    let pull_pipe = DebugOp::new("channel", pull_pipe);

    push_pipe.push(350).await;
    push_pipe.push(650).await;

    let mut comp = StaticComp::new(pull_pipe, push_pipe);
    for _ in 0_usize..10 {
        comp.tick_moveop().await;
    }

    Ok(())
}

#[tokio::test]
pub async fn test_cycle_handoff() -> Result<(), String> {

    let ( push_pipe, pull_pipe ) = handoff_op::<DF, usize>();
    let mut push_pipe = push_pipe;

    let pull_pipe = MapFilterMoveOp::<_, _, usize>::new(pull_pipe, |x: usize| Some(x + 1));
    let pull_pipe = DebugOp::new("handoff", pull_pipe);

    push_pipe.push(150).await;

    let mut comp = StaticComp::new(pull_pipe, push_pipe);
    for _ in 0_usize..10 {
        comp.tick_moveop().await;
    }

    Ok(())
}
