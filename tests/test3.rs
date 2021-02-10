use spinach::op2::{ MoveNext, MovePushOp, DebugOp, MapFilterMoveOp, channel_op, handoff_op };

#[tokio::test]
pub async fn test_cycle_channel() -> Result<(), String> {

    let ( push_pipe, pull_pipe ) = channel_op::<usize>(1);
    let mut push_pipe = push_pipe;

    let pull_pipe = MapFilterMoveOp::<_, _, usize>::new(pull_pipe, |x: usize| Some(x + 1));
    let pull_pipe = DebugOp::new("channel", pull_pipe);
    let mut pull_pipe = pull_pipe;

    push_pipe.push(350).await;
    for _ in 0_usize..10 {
        if let Some(item) = MoveNext::new(&mut pull_pipe).await {
            push_pipe.push(item).await;
        }
    }
    Ok(())
}

#[tokio::test]
pub async fn test_cycle_handoff() -> Result<(), String> {

    let ( push_pipe, pull_pipe ) = handoff_op::<usize>();
    let mut push_pipe = push_pipe;

    let pull_pipe = MapFilterMoveOp::<_, _, usize>::new(pull_pipe, |x: usize| Some(x + 1));
    let pull_pipe = DebugOp::new("handoff", pull_pipe);
    let mut pull_pipe = pull_pipe;

    push_pipe.push(150).await;
    for _ in 0_usize..10 {
        if let Some(item) = MoveNext::new(&mut pull_pipe).await {
            push_pipe.push(item).await;
        }
    }
    Ok(())
}
