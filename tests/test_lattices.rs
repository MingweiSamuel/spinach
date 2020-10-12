use std::collections::{ HashMap, HashSet };

use spinach::Lattice;
use spinach::merge::{ MaxMerge, MinMerge, UnionMerge, MapUnionMerge, DominatingPairMerge };

#[test]
fn test_counter() {
    let mut counter: Lattice<i32, MaxMerge> = Lattice::default();
    counter.merge_in(5_i32.into());
    counter.merge_in(4_i32.into());
    counter.merge_in(6_i32.into());
    counter.merge_in(3_i32.into());
    assert_eq!(6_i32, counter.into_reveal());
}

#[test]
fn test_opt_counter() {
    let mut counter: Lattice<Option<i32>, MaxMerge> = Lattice::default();
    counter.merge_in(None.into());
    assert_eq!(&None, counter.reveal());
    counter.merge_in(Some(-9_999_999_i32).into());
    assert_eq!(&Some(-9_999_999_i32), counter.reveal());
    counter.merge_in(Some(0_i32).into());
    assert_eq!(&Some(0_i32), counter.reveal());
    counter.merge_in(Some(-100_i32).into());
    assert_eq!(&Some(0_i32), counter.reveal());
}

#[test]
fn test_items() {
    let mut items: Lattice<HashSet<i32>, UnionMerge> = Lattice::default();
    items.merge_in([ 1, 2, 3 ].iter().copied().collect());
    items.merge_in([ 2, 3, 4 ].iter().copied().collect());
    assert_eq!([ 1, 2, 3, 4 ].iter().copied().collect::<HashSet<i32>>(), items.into_reveal());
}

#[test]
fn test_vclock() {
    type VectorClock = Lattice<HashMap<&'static str, Lattice<u64, MaxMerge>>, MapUnionMerge>;
    let mut vclock: VectorClock = Lattice::default();
    vclock.merge_in(
        vec![
            ("Norfolk", 80_u64.into()),
            ("Oakland", 50_u64.into()),
            ("SanFran", 20_u64.into()),
        ]
        .into_iter().collect());
    vclock.merge_in(
        vec![
            ("Norfolk", 90_u64.into()),
            ("Norwalk", 50_u64.into()),
            ("SanFran", 10_u64.into()),
        ]
        .into_iter().collect());
    println!("VClock:");
    for (k, v) in vclock.into_reveal() {
        println!("- {}: {}", k, v.into_reveal());
    }
    println!();
    // TODO: assert.
}

#[test]
fn test_lexico_str() {
    let mut lexico_str: Lattice<&'static str, MinMerge> = "qux".into();
    lexico_str.merge_in("foo".into());
    lexico_str.merge_in("bar".into());
    lexico_str.merge_in("zab".into());
    assert_eq!("bar", lexico_str.into_reveal());
}

#[test]
fn test_lexico_tuple() {
    let mut tup: Lattice<(Lattice<i32, MaxMerge>, Lattice<i32, MinMerge>), DominatingPairMerge> = Lattice::default();
    println!("0 ({}, {})", tup.reveal().0.reveal(), tup.reveal().1.reveal());
    tup.merge_in((0.into(), 0.into()).into());
    println!("1 ({}, {})", tup.reveal().0.reveal(), tup.reveal().1.reveal());
    tup.merge_in((1.into(), 1.into()).into());
    println!("2 ({}, {})", tup.reveal().0.reveal(), tup.reveal().1.reveal());
    tup.merge_in((2.into(), 5.into()).into());
    println!("3 ({}, {})", tup.reveal().0.reveal(), tup.reveal().1.reveal());
    tup.merge_in((2.into(), 2.into()).into());
    println!("4 ({}, {})", tup.reveal().0.reveal(), tup.reveal().1.reveal());
}
