
// pub trait Set<T>: SetRead<T> + SetWrite<T> {}
// impl <S, T> Set<T> for S where S: SetRead<T> + SetWrite<T> {}



// pub trait SetRead<T> {
//     type MapOutput<U>; // GAT (does not work, not allowed to have <U> parameter).

//     fn map<U, F>(&self, f: F) -> Self::MapOutput<U>
//     where
//         F: Fn(T) -> U;
// }




pub trait SetRead<'a, T> {
    fn map<U>(&'a self, f: impl Fn(T) -> U) -> <Self as SetReadMap<U>>::Output
    where
        T: 'a,
        Self: SetReadMap<U>;
}
pub trait SetReadMap<U> {
    type Output: std::iter::FromIterator<U>;
}









impl <'a, T> SetRead<'a, &'a T> for std::collections::HashSet<T>
where
    T: Eq + std::hash::Hash,
{
    fn map<U>(&'a self, f: impl Fn(&'a T) -> U) -> <Self as SetReadMap<U>>::Output
    where
        T: 'a,
        Self: SetReadMap<U>,
    {
        self.into_iter()
            .map(f)
            .collect()
    }
}
impl <T, U> SetReadMap<U> for std::collections::HashSet<T>
where
    U: Eq + std::hash::Hash,
{
    type Output = std::collections::HashSet<U>;
}




// impl <T> SetRead<&T> for std::collections::BTreeSet<T>
// where
//     T: Eq + Ord,
// {
//     fn map<U>(&self, f: impl Fn(&T) -> U) -> <Self as SetReadMap<U>>::Output
//     where
//         Self: SetReadMap<U>,
//     {
//         self.into_iter()
//             .map(f)
//             .collect()
//     }
// }
// impl <T, U> SetReadMap<U> for std::collections::BTreeSet<T>
// where
//     U: Eq + Ord,
// {
//     type Output = std::collections::BTreeSet<U>;
// }




// impl <K, V> SetRead<( &K, &V )> for std::collections::BTreeMap<K, V>
// where
//     K: Eq + Ord,
// {
//     fn map<U>(&self, f: impl Fn(( &K, &V )) -> U) -> <Self as SetReadMap<U>>::Output
//     where
//         Self: SetReadMap<U>,
//     {
//         self.into_iter()
//             .map(f)
//             .collect()
//     }
// }
// impl <K, V, U> SetReadMap<U> for std::collections::BTreeMap<K, V>
// where
//     U: Eq + Ord,
// {
//     type Output = std::collections::BTreeSet<U>;
// }











pub trait SetWrite<T> {

}
