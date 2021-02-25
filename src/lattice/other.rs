use std::cmp::Ordering;

use super::Lattice;

// /// Wrap an existing lattice in `Option`, where None is smaller than all other elements.
// pub struct Optional<F: Lattice> {
//     _phantom: std::marker::PhantomData<F>,
// }
// impl<F: Lattice> Lattice for Optional<F> {
//     type Domain = Option<F::Domain>;

//     fn merge_in(val: &mut Self::Domain, delta: Self::Domain) {
//         *val = val.or(delta);
//         not_implemented!()
//     }

//     fn partial_cmp(a: &Self::Domain, b: &Self::Domain) -> Option<Ordering> {
//         match a {
//             None => {
//                 match b {
//                     None => Some(Ordering::Equal),
//                     Some(_) => Some(Ordering::Less),
//                 }
//             }
//             Some(a) => {
//                 match b {
//                     None => Some(Ordering::Greater),
//                     Some(b) => F::partial_cmp(a, b),
//                 }
//             }
//         }
//     }
// }

/// Mingwei's weird semilattice.
/// Lattice is defined as, given signed integers A and B, take the value in the
/// range [A, B] (or [B, A]) which is closest to zero.
/// (Note that in general this will be A, B, or zero).
pub struct RangeToZeroI32;
impl Lattice for RangeToZeroI32 {
    type Domain = i32;

    fn merge_in(val: &mut i32, delta: i32) {
        if val.signum() != delta.signum() {
            *val = 0;
        } else if val.abs() > delta.abs() {
            *val = delta
        }
    }

    fn partial_cmp(a: &i32, b: &i32) -> Option<Ordering> {
        if a.signum() != b.signum() {
            None
        } else {
            let less = a.abs().cmp(&b.abs());
            Some(less.reverse())
        }
    }
}
