use std::cmp::Ordering;

use super::Merge;

pub struct DominatingPair<AF, BF>
where
    AF: Merge,
    BF: Merge,
{
    _phantom: std::marker::PhantomData<(AF, BF)>,
}

impl<AF, BF> Merge for DominatingPair<AF, BF>
where
    AF: Merge,
    BF: Merge,
{
    type Domain = (<AF as Merge>::Domain, <BF as Merge>::Domain);

    fn merge_in(val: &mut Self::Domain, delta: Self::Domain) {
        match AF::partial_cmp(&val.0, &delta.0) {
            None => {
                AF::merge_in(&mut val.0, delta.0);
                BF::merge_in(&mut val.1, delta.1);
            }
            Some(Ordering::Equal) => {
                BF::merge_in(&mut val.1, delta.1);
            }
            Some(Ordering::Less) => {
                *val = delta;
            }
            Some(Ordering::Greater) => {}
        }
    }

    fn partial_cmp(val: &Self::Domain, delta: &Self::Domain) -> Option<Ordering> {
        AF::partial_cmp(&val.0, &delta.0).or_else(|| BF::partial_cmp(&val.1, &delta.1))
    }

    fn remainder(val: &mut Self::Domain, delta: Self::Domain) -> bool {
        match AF::partial_cmp(&val.0, &delta.0) {
            None => {
                AF::merge_in(&mut val.0, delta.0);
                BF::remainder(&mut val.1, delta.1);
                false
            }
            Some(Ordering::Equal) => {
                BF::remainder(&mut val.1, delta.1);
                false
            }
            Some(Ordering::Less) => {
                *val = delta;
                false
            }
            Some(Ordering::Greater) => true,
        }
    }
}
