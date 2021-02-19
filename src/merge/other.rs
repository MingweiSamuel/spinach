use std::cmp::Ordering;

use super::Merge;

/// Mingwei's weird semilattice.
/// Merge is defined as, given signed integers A and B, take the value in the
/// range [A, B] (or [B, A]) which is closest to zero.
/// (Note that in general this will be A, B, or zero).
pub struct RangeToZeroI32;
impl Merge for RangeToZeroI32 {
    type Domain = i32;

    fn merge_in(val: &mut i32, delta: i32) {
        if val.signum() != delta.signum() {
            *val = 0;
        }
        else if val.abs() > delta.abs() {
            *val = delta
        }
    }

    fn partial_cmp(val: &i32, delta: &i32) -> Option<Ordering> {
        if val.signum() != delta.signum() {
            None
        }
        else {
            let less = val.abs().cmp(&delta.abs());
            Some(less.reverse())
        }
    }
}
