# Constant evaluation

Constant evaluation is the process of computing the result of
[expressions] during compilation. Only a subset of all expressions
can be evaluated at compile-time.

## Constant expressions

Certain forms of expressions, called constant expressions, can be evaluated at
compile time. In [const contexts](#const-context), these are the only allowed
expressions, and are always evaluated at compile time. In other places, such as
[`let` statements](statements.html#let-statements), constant expressions *may*
be, but are not guaranteed to be, evaluated at compile time. Behaviors such as
out of bounds [array indexing] or [overflow] are compiler errors if the value
must be evaluated at compile time (i.e. in const contexts). Otherwise, these
behaviors are warnings, but will likely panic at run-time.

The following expressions are constant expressions, so long as any operands are
also constant expressions and do not cause any [`Drop::drop`][destructors] calls
to be ran.

* [Literals].
* [Paths] to [functions](items/functions.html) and constants.
  Recursively defining constants is not allowed.
* [Tuple expressions].
* [Array expressions].
* [Struct] expressions.
* [Enum variant] expressions.
* [Block expressions], including `unsafe` blocks, which only contain items and
  possibly a constant tail expression.
* [Field] expressions.
* Index expressions, [array indexing] or [slice] with a `usize`.
* [Range expressions].
* [Closure expressions] which don't capture variables from the environment.
* Built in [negation], [arithmetic, logical], [comparison] or [lazy boolean]
  operators used on integer and floating point types, `bool` and `char`.
* Shared [borrow]s, except if applied to a type with [interior mutability].
* The [dereference operator].
* [Grouped] expressions.
* [Cast] expressions, except pointer to address and
  function pointer to address casts.
* Calls of const functions and const methods

## Const context

A _const context_ is one of the following:

* [Array type length expressions]
* Repeat expression length expressions
* The initializer of
  * [constants]
  * [statics]
  * [enum discriminants]

[array type length expressions]: types/array.html
[enum discriminants]: items/enumerations.html#custom-discriminant-values-for-field-less-enumerations
[constants]: items/constant-items.html
[statics]: items/static-items.html
[expressions]: expressions.html
[array indexing]:       expressions/array-expr.html#array-and-slice-indexing-expressions
[overflow]:             expressions/operator-expr.html#overflow
[destructors]:          destructors.html
[literals]:             expressions/literal-expr.html
[paths]:                expressions/path-expr.html
[tuple expressions]:    expressions/tuple-expr.html
[array expressions]:    expressions/array-expr.html
[struct]:               expressions/struct-expr.html
[enum variant]:         expressions/enum-variant-expr.html
[block expressions]:    expressions/block-expr.html
[field]:                expressions/field-expr.html
[array indexing]:       expressions/array-expr.html#array-and-slice-indexing-expressions
[slice]:                types/slice.html
[range expressions]:    expressions/range-expr.html
[closure expressions]:  expressions/closure-expr.html
[negation]:             expressions/operator-expr.html#negation-operators
[arithmetic, logical]:  expressions/operator-expr.html#arithmetic-and-logical-binary-operators
[comparison]:           expressions/operator-expr.html#comparison-operators
[lazy boolean]:         expressions/operator-expr.html#lazy-boolean-operators
[borrow]:               expressions/operator-expr.html#borrow-operators
[interior mutability]:  interior-mutability.html
[dereference operator]: expressions/operator-expr.html#the-dereference-operator
[grouped]:              expressions/grouped-expr.html
[cast]:                 expressions/operator-expr.html#type-cast-expressions
