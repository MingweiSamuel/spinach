#![feature(generic_associated_types)]
#![allow(dead_code)]

/*
    function cartProd(a, b) {
        if (0 >= a.length) return [];
        const [ lhs, ...aRest ] = a;
        return [ cartProdHelper(lhs, b), cartProd(aRest, b) ];
    }
    function cartProdHelper(lhs, b) {
        if (0 >= b.length) return [];
        const [ rhs, ...bRest ] = b;
        return [ [ lhs, rhs ], cartProdHelper(lhs, bRest) ];
    }

    console.log(JSON.stringify(cartProd([ 1, 2, 3 ], [ 'a', 'b', 'c', 'd' ]), null, 2));
*/

trait CartProd {
    type Result;
}
trait CartProdHelper {
    type Result;
}

impl<B> CartProd for ((), B) {
    type Result = ();
}
impl<Lhs> CartProdHelper for (Lhs, ()) {
    type Result = ();
}

impl<Lhs, ARest, B> CartProd for ((Lhs, ARest), B)
where
    (Lhs, B): CartProdHelper,
    (ARest, B): CartProd,
{
    type Result = (
        <(Lhs, B) as CartProdHelper>::Result,
        <(ARest, B) as CartProd>::Result
    );
}
impl<Lhs, Rhs, BRest> CartProdHelper for (Lhs, (Rhs, BRest))
where
    (Lhs, BRest): CartProdHelper,
{
    type Result = (
        (Lhs, Rhs),
        <(Lhs, BRest) as CartProdHelper>::Result
    );
}

type TypeListA = (u8, (u16, (u32, ())));
type TypeListB = (i8, (i16, (i32, (i64, ()))));

type MyCartProd = <(TypeListA, TypeListB) as CartProd>::Result;

const X: MyCartProd = (
    ((0_u8, 0_i8), ((0_u8, 0_i16), ((0_u8, 0_i32), ((0_u8, 0_i64), ())))),
    (
        ((0_u16, 0_i8), ((0_u16, 0_i16), ((0_u16, 0_i32), ((0_u16, 0_i64), ())))),
        (
            ((0_u32, 0_i8), ((0_u32, 0_i16), ((0_u32, 0_i32), ((0_u32, 0_i64), ())))),
            ()
        )
    )
);


// trait CartesianProduct {
//     type CartesianProduct;
// }
// trait CartesianProductHelper<C: CartesianProduct> {
//     type CartesianProductHelper;
// }

// impl CartesianProduct for () {
//     type CartesianProduct = ();
// }
// impl<C: CartesianProduct> CartesianProductHelper<C> for () {
//     type CartesianProductHelper = ();
// }

// impl<Head: CartesianProductHelper<Tail>, Tail: CartesianProduct> CartesianProduct for (Head, Tail) {
//     type CartesianProduct = (Head::CartesianProductHelper, Tail::CartesianProduct);
// }
// impl<Head, Tail: CartesianProduct> CartesianProductHelper<C>

// type Z = <(u32, (String, [usize; 5])) as CartesianProduct>::CartesianProduct;


// use std::collections::HashMap;

// // use tuple_list::{ TupleList, tuple_list, tuple_list_type };

// use spinach::lattice::LatticeRepr;

// // pub trait LatReprList

// // pub struct LatticeReprList<Head: LatticeRepr, Tail: TupleList>

// // pub struct DispatchTable<>



// pub trait LatticeFn {
//     type In:  LatticeRepr;
//     type Out: LatticeRepr;
//     fn run<'a>(arg: Self::In) -> Self::Out;
// }

// pub trait LatticeReprList {
//     type AsVec;
//     fn as_vec() -> Self::AsVec;
// }
// impl LatticeReprList for () {
//     type AsVec = ();
//     fn as_vec() -> Self::AsVec {
//         ()
//     }
// }
// impl<Head: LatticeRepr, Tail: LatticeReprList> LatticeReprList for (Head, Tail) {
//     type AsVec = (Vec<Head>, Tail::AsVec);
//     fn as_vec() -> Self::AsVec {
//         (Vec::new(), Tail::as_vec())
//     }
// }


// pub struct DispatchTable<T: LatticeReprList> {
//     table: T::AsVec,
// }
// impl<T: LatticeReprList> DispatchTable<T> {
//     pub fn new() -> Self {
//         Self {
//             table: T::as_vec(),
//         }
//     }
// }


// // Example
// /*
// Split points:
// LatX
// LatY
// LatZ

// Functions:
// FnA: LatX -> LatY
// FnB: LatY -> LatZ
// FnC: LatZ -> LatX

// Split points type: (LatX, LatY, LatZ)
// Functions type:
//     (
//         (FnA, ????


//     )

// */



fn main() {

}
