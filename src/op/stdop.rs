use std::future;
use std::io::Write;

use crate::flow::*;

use super::*;

/// An Op which writes to stdout.
pub struct StdOutOp<F: Flow, T: AsRef<[u8]>> {
    _phantom: std::marker::PhantomData<(F, T)>,
}

impl<F: Flow, T: AsRef<[u8]>> StdOutOp<F, T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<F: Flow, T: AsRef<[u8]>> Default for StdOutOp<F, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: Flow, T: AsRef<[u8]>> Op for StdOutOp<F, T> {}

impl<F: Flow, T: AsRef<[u8]>> PushOp for StdOutOp<F, T> {
    type Inflow = F;
    type Indomain<'p> = T;

    type Feedback<'s> = future::Ready<()>;
 
    fn push<'s, 'p>(&'s mut self, item: Self::Indomain<'p>) -> Self::Feedback<'s> {
        let bytes = item.as_ref();
        std::io::stdout().write_all(bytes).expect("Failed to write to stdout.");
        future::ready(())
    }
}
