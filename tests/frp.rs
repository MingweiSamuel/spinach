use std::collections::{ BTreeSet };
use std::sync::mpsc;
use std::thread;

use spinach::semilattice::Semilattice;
use spinach::traits::{ SetUnion, SemilatticeMorphismFn, UnaryFn };


#[test]
fn test_frp() {
    type Triple = (&'static str, usize, &'static str);
    let (sender, receiver) = mpsc::channel::<Triple>();

    thread::spawn(move || {
        let mut all_el: Semilattice<SetUnion<BTreeSet<Triple>>> = Semilattice::new(BTreeSet::new());
        while let Ok(el) = receiver.recv() {
            let mut delta = BTreeSet::new();
            delta.insert(el);

            all_el.merge_in(delta);

            let all_el_clone = all_el.clone();
        }
        println!("Done!");
    });

    sender.send(("hello", 5, "world"));
    sender.send(("hello", 5, "world"));
    sender.send(("asdf",  5, "world"));
    sender.send(("hello", 5, "world"));
    sender.send(("hello", 5, "world"));
    sender.send(("hello", 5, "world"));
    sender.send(("hello", 5, "world"));
}
