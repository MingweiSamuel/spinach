What do we want this to look like?

# Lattice Type
```
type MyVectorClockVersionedHashMap =
    MapUnion<Key,
        DominatingPair<
            MapUnion<Id, Max<usize>>,
            Value
        >
    >
;
```