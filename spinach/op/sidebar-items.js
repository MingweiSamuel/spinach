initSidebarItems({"constant":[["UDP_BUFFER","Buffer size for the [`UdpPullOp`]. NOTE that any packets longer than this will be truncated!"]],"fn":[["channel_op","Create a connected sending and receiving channel pair, [`ChannelPushOp`] and [`ChannelPullOp`]."],["handoff_op","Create a connected sending and receiving handoff pair, [`HandoffPushOp`] and [`HandoffPullOp`]."],["udp_op","Create a pull and push pair from a [`UdpSocket`]."]],"struct":[["AsRefOp","An Op for converting an owned flow into a reference flow via [`AsRef`]."],["BatchingOp","An op which releases batches of values on a timer interval."],["BlockingIntervalOp","An op which releases individual values on a timer interval."],["ChannelPullOp","The receiving (pull) half of a channel."],["ChannelPushOp","The sending (push) half of a channel."],["CloneOp","An Op for converting a ref flow into an owned flow via [`Clone`]."],["DebugOp","An Op which logs each passing element to stdout, for debugging."],["HandoffPullOp","The receiving (pull) half of a handoff."],["HandoffPushOp","The sending (push) half of a handoff."],["LatticeOp","A state-accumulating lattice op."],["LeakyIntervalOp","An op which releases individual values on a timer interval."],["MapFilterMoveOp","Map-Filter op for owned->owned values."],["MapFlattenMoveOp","Map-Flatten op for owned->owned values."],["MapFoldRefOp","Map-Fold op for ref->owned values."],["MapRefRefOp","Map op for ref->ref values."],["MergeOp","An Op which receives from two upstream ops."],["MonotonicFilterRefOp","A specific type of monotonic mapping Op."],["NullOp","An Op which does nothing. Supports both [`Df`] and [`Rx`]."],["ReferenceOp","An Op for converting an owned flow into a reference flow. Simply takes the reference of the owned value."],["RendezvousOp","An op which rendezvous dataflow values to a reactive value."],["SplitOp","An Op which, for each element, copies it and pushes to both downstream ops. Note: [`Copy`] is lightweight and implemented for references and simple primitives. It is not [`Clone`]."],["StdOutOp","An Op which writes to stdout."],["ToOwnedOp","An Op for converting a ref flow into an owned flow via [`ToOwned`]."],["UdpPullOp","The receving (pull) side of a UDP connection."],["UdpPushOp","The sending (push) side of a UDP connection."]],"trait":[["Op","An empty trait indicating a struct can be used as an Op."],["PullOp","A pull-based op, specifying an Outflow domain/flow type."],["PushOp","A push-based op, specifying an Inflow domain/flow type."]]});