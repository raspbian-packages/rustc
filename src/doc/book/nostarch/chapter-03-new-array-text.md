Please add this text at the end of The Array Type section, just before the Accessing Array Elements subsection starts on page 41.

Writing an array's type is done with square brackets containing the type of
each element in the array followed by a semicolon and the number of elements in
the array, like so:

```rust
let a: [i32; 5] = [1, 2, 3, 4, 5];
```

Here, `i32` is the type of each element. After the semicolon, the number `5`
indicates the element contains five items.

The way an array's type is written looks similar to an alternative syntax for
initializing an array: if you want to create an array that contains the same
value for each element, you can specify the initial value, then a semicolon,
then the length of the array in square brackets as shown here:

```rust
let a = [3; 5];
```

The array named `a` will contain 5 elements that will all be set to the value
`3` initially. This is the same as writing `let a = [3, 3, 3, 3, 3];` but in a
more concise way.
