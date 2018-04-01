# Special types and traits

Certain types and traits that exist in [the standard library] are known to the
Rust compiler. This chapter documents the special features of these types and
traits.

## `Box<T>`

[`Box<T>`] has a few special features that Rust doesn't currently allow for user
defined types.

* The [dereference operator] for `Box<T>` produces a place which can be moved
  from. This means that the `*` operator and the destructor of `Box<T>` are
  built-in to the language.
* [Methods] can take `Box<Self>` as a receiver.
* A trait may be implemented for `Box<T>` in the same crate as `T`, which the
  [orphan rules] prevent for other generic types.

## `UnsafeCell<T>`

[`std::cell::UnsafeCell<T>`] is used for [interior mutability]. It ensures that
the compiler doesn't perform optimisations that are incorrect for such types.
It also ensures that [`static` items] which have a type with interior
mutability aren't placed in memory marked as read only.

## `PhantomData<T>`

[`std::marker::PhantomData<T>`] is a zero-sized, minimum alignment, type that
is considered to own a `T` for the purposes of [variance], [drop check] and
[auto traits](#auto-traits).

## Operator Traits

The traits in [`std::ops`] and [`std::cmp`] are used to overload [operators],
[indexing expressions] and [call expressions].

## `Deref` and `DerefMut`

As well as overloading the unary `*` operator, [`Deref`] and [`DerefMut`] are
also used in [method resolution] and [deref coercions].

## `Drop`

The [`Drop`] trait provides a [destructor], to be run whenever a value of this
type is to be destroyed.

## `Copy`

The [`Copy`] trait changes the semantics of a type implementing it. Values
whose type implements `Copy` are copied rather than moved upon assignment.
`Copy` cannot be implemented for types which implement `Drop`, or which have
fields that are not `Copy`. `Copy` is implemented by the compiler for

* [Numeric types]
* `char` and `bool`
* [Tuples] of `Copy` types
* [Arrays] of `Copy` types
* [Shared references]
* [Raw pointers]
* [Function pointers] and [function item types]

## `Clone`

The [`Clone`] trait is a supertrait of `Copy`, so it also needs compiler
generated implementations. It is implemented by the compiler for the following
types:

* Types with a built-in `Copy` implementation (see above)
* [Tuples] of `Clone` types
* [Arrays] of `Clone` types

## `Send`

The [`Send`] trait indicates that a value of this type is safe to send from one
thread to another.

## `Sync`

The [`Sync`] trait indicates that a value of this type is safe to share between
multiple threads. This trait must be implemented for all types used in
immutable [`static` items].

## Auto traits

The [`Send`], [`Sync`], [`UnwindSafe`] and [`RefUnwindSafe`] traits are _auto
traits_. Auto traits have special properties.

First, auto traits are automatically implemented using the following rules:

* `&T`, `&mut T`, `*const T`, `*mut T`, `[T; n]` and `[T]` implement the trait
  if `T` does.
* Function item types and function pointers automatically implement the trait.
* Structs, enums, unions and tuples implement the trait if all of their fields
  do.
* Closures implement the trait if the types of all of their captures do. A
  closure that captures a `T` by shared reference and a `U` by value implements
  any auto traits that both `&T` and `U` do.

Auto traits can also have negative implementations, shown as `impl !AutoTrait
for T` in the standard library documentation, that override the automatic
implementations. For example `*mut T` has a negative implementation of `Send`,
and so `*mut T` and `(*mut T,)` are not `Send`. Finally, auto traits may
be added as a bound to any [trait object]\: `Box<Debug + Send + UnwindSafe>` is
a valid type.

## `Sized`

The [`Sized`] trait indicates that the size of this type is known at
compile-time; that is, it's not a [dynamically sized type]. [Type parameters]
are `Sized` by default. `Sized` is always implemented automatically by the
compiler, not by [implementation items].

[`Box<T>`]: ../std/boxed/struct.Box.html
[`Clone`]: ../std/clone/trait.Clone.html
[`Copy`]: ../std/marker/trait.Copy.html
[`Deref`]: ../std/ops/trait.Deref.html
[`DerefMut`]: ../std/ops/trait.DerefMut.html
[`Drop`]: ../std/ops/trait.Drop.html
[`RefUnwindSafe`]: ../std/panic/trait.RefUnwindSafe.html
[`Send`]: ../std/marker/trait.Send.html
[`Sized`]: ../std/marker/trait.Sized.html
[`std::cell::UnsafeCell<T>`]: ../std/cell/struct.UnsafeCell.html
[`std::cmp`]: ../std/cmp/index.html
[`std::marker::PhantomData<T>`]: ../std/marker/struct.PhantomData.html
[`std::ops`]: ../std/ops/index.html
[`UnwindSafe`]: ../std/panic/trait.UnwindSafe.html
[`Sync`]: ../std/marker/trait.Sync.html

[Arrays]: types.html#array-and-slice-types
[call expressions]: expressions/call-expr.html
[deref coercions]: type-coercions.html#coercion-types
[dereference operator]: expressions/operator-expr.html#the-dereference-operator
[destructor]: destructors.html
[drop check]: ../nomicon/dropck.html
[dynamically sized type]: dynamically-sized-types.html
[Function pointers]: types.html#function-pointer-types
[function item types]: types.html#function-item-types
[implementation items]: items/implementations.html
[indexing expressions]: expressions/array-expr.html#array-and-slice-indexing-expressions
[interior mutability]: interior-mutability.html
[Numeric types]: types.html#numeric-types
[Methods]: items/traits.html#associated-functions-and-methods
[method resolution]: expressions/method-call-expr.html
[operators]: expressions/operator-expr.html
[orphan rules]: items/implementations.html#trait-implementation-coherence
[Raw pointers]: types.html#raw-pointers-const-and-mut
[`static` items]: items/static-items.html
[Shared references]: types.html#shared-references-
[the standard library]: ../std/index.html
[trait object]: types.html#trait-objects
[Tuples]: types.html#tuple-types
[Type parameters]: types.html#type-parameters
[variance]: ../nomicon/subtyping.html
