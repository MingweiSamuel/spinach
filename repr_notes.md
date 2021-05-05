What do we want this to look like?

# Lattice Type
```rs
type MyVectorClockVersionedHashMap =
    MapUnion<Key,
        DominatingPair<
            MapUnion<Id, Max<usize>>,
            Max<String>
        >
    >
;
```

```rs
SetUnion<String> -> SetUnion<u32>
```