# Standard Library

The spec files make heavy use of a set of utility functions defined in
[`0-stdlib.watsup`](https://github.com/pacokwon/nano-p4-spec/blob/main/0-stdlib.watsup).
These are not specific to Nano-P4. Rather, they are generic building blocks for
working with sequences, sets, and maps in P4-SpecTec.

This page is a reference you can return to as you read through Sections 3 and 4.
You do not need to memorize everything here; skim it once so the names are
familiar, then come back when you need a reminder.

## Sequences

A sequence (written `X*`) is an ordered list of elements. The empty sequence is
`eps`, and `::` is the cons operator.

```spectec
builtin dec $rev_<X>(X*) : X*
```

`$rev_` reverses a sequence.

```spectec
builtin dec $distinct_<K>(K*) : bool
```

`$distinct_` returns `true` if all elements in the sequence are unique. It is
used to enforce no-duplicate-name constraints, for example checking that a
parameter list does not repeat a name.

```spectec
builtin dec $assoc_<X, Y>(X, (X, Y)*) : Y?
```

`$assoc_` looks up a key in an association list (a sequence of pairs) and
returns the associated value, or `eps` if not found.

```spectec
dec $repeat_<X>(X, nat) : X*
```

`$repeat_` produces a sequence of `nat` copies of a value.

```spectec
dec $exists_(bool*) : bool
dec $forall_(bool*) : bool
```

`$exists_` returns `true` if at least one element is `true`. `$forall_` returns
`true` if all elements are `true`.

## Sets

A set is written as `` `{ K* } ``. a collection of unique keys with no ordering.

```spectec
dec $empty_set<K> : set<K>
dec $in_set<K>(set<K>, K) : bool
```

`$empty_set` produces an empty set. `$in_set` checks whether a key is a member
of a set.

```spectec
builtin dec $intersect_set<K>(set<K>, set<K>) : set<K>
builtin dec $union_set<K>(set<K>, set<K>)     : set<K>
builtin dec $unions_set<K>(set<K>*)            : set<K>
builtin dec $diff_set<K>(set<K>, set<K>)       : set<K>
builtin dec $sub_set<K>(set<K>, set<K>)        : bool
builtin dec $eq_set<K>(set<K>, set<K>)         : bool
```

Standard set operations: intersection, union, union of a sequence of sets,
difference, subset check, and equality.

## Maps

A map is a set of key-value pairs, written `{ (K : V)* }`.

```spectec
dec $empty_map<K, V> : map<K, V>
```

`$empty_map` produces an empty map.

```spectec
dec $dom_map<K, V>(map<K, V>) : set<K>
dec $codom_map<K, V>(map<K, V>) : set<V>
```

`$dom_map` returns the set of keys. `$codom_map` returns the set of values.
`$dom_map` appears frequently in the spec to check whether a name is already
bound before adding it.

```spectec
builtin dec $find_map<K, V>(map<K, V>, K) : V?
builtin dec $find_maps<K, V>(map<K, V>*, K) : V?
```

`$find_map` looks up a key in a single map and returns the value, or `eps` if
not found. `$find_maps` searches a _sequence_ of maps from left to right,
returning the first match. This is used for variable lookup across a stack of
frames.

```spectec
builtin dec $add_map<K, V>(map<K, V>, K, V) : map<K, V>
builtin dec $update_map<K, V>(map<K, V>, K, V) : map<K, V>
```

`$add_map` inserts a new key-value pair. The spec always checks that the key is
not already present before calling `$add_map`, so it is effectively a
no-overwrite insert. `$update_map` updates the value for an existing key. One
example usage of this function is when writing to a variable that is already in
scope.
