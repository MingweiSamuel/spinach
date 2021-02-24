initSidebarItems({"fn":[["channel_op","Create a connected sending and receiving channel pair, [`ChannelPushOp`] and [`ChannelPullOp`]."],["handoff_op","Create a connected sending and receiving handoff pair, [`HandoffPushOp`] and [`HandoffPullOp`]."],["sleep_yield_now","An async function which puts the current task to sleep. Unlike [`tokio::task::yield_now`], this marks the current task as not ready, so it will remain asleep until the task is awoken by an event."]],"mod":[["flow",""]],"struct":[["BatchingOp","An op which releases batches of values on a timer interval."],["BlockingIntervalOp","An op which releases individual values on a timer interval."],["ChannelPullOp","The receiving (pull) half of a channel."],["ChannelPushOp","The sending (push) half of a channel."],["CloneOp","An Op for converting a ref flow into an owned flow via [`Clone`]."],["DebugOp","An Op which logs each passing element to stdout, for debugging."],["DynComp","A computation node with a single pull and dynamically many push ends."],["HandoffPullOp","The receiving (pull) half of a handoff."],["HandoffPushOp","The sending (push) half of a handoff."],["LatticeOp","A state-accumulating lattice op."],["LeakyIntervalOp","An op which releases individual values on a timer interval."],["MapFilterMoveOp",""],["MapFlattenMoveOp","Map-Flatten op for with owned->owned values."],["MapFoldRefOp","Map-Fold op for ref->owned values."],["MonotonicFilterRefOp","A specific type of monotonic mapping Op for [`Rx`] pipelines."],["MoveNext","Helper future struct for getting a value from [`MovePullOp`]s."],["NullOp","An Op which does nothing. Supports both [`Df`] and [`Rx`]."],["ReferenceOp","An Op for converting an owned flow into a reference flow. Simply takes the reference of the owned value."],["StaticComp","A computation node with a single pull end and a single push end."]],"trait":[["MovePullOp","Pull-based op for owned values."],["MovePushOp","Push-based op for owned values."],["Op","An empty trait indicating a struct can be used as an Op."],["PullOp","A pull-based op, specifying an Outflow domain/flow type."],["PushOp","A push-based op, specifying an Inflow domain/flow type."],["RefPullOp","Pull-based op for reference values."],["RefPushOp","Push-based op for reference values."]]});