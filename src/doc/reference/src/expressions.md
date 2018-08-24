# Expressions

> **<sup>Syntax</sup>**  
> _Expression_ :  
> &nbsp;&nbsp; &nbsp;&nbsp; [_LiteralExpression_]  
> &nbsp;&nbsp; | [_PathExpression_]  
> &nbsp;&nbsp; | [_BlockExpression_]  
> &nbsp;&nbsp; | [_OperatorExpression_]  
> &nbsp;&nbsp; | [_GroupedExpression_]  
> &nbsp;&nbsp; | [_ArrayExpression_]  
> &nbsp;&nbsp; | [_IndexExpression_]  
> &nbsp;&nbsp; | [_TupleExpression_]  
> &nbsp;&nbsp; | [_TupleIndexingExpression_]  
> &nbsp;&nbsp; | [_StructExpression_]  
> &nbsp;&nbsp; | [_EnumerationVariantExpression_]  
> &nbsp;&nbsp; | [_CallExpression_]  
> &nbsp;&nbsp; | [_MethodCallExpression_]  
> &nbsp;&nbsp; | [_FieldExpression_]  
> &nbsp;&nbsp; | [_ClosureExpression_]  
> &nbsp;&nbsp; | [_LoopExpression_]  
> &nbsp;&nbsp; | [_ContinueExpression_]  
> &nbsp;&nbsp; | [_BreakExpression_]  
> &nbsp;&nbsp; | [_RangeExpression_]  
> &nbsp;&nbsp; | [_IfExpression_]  
> &nbsp;&nbsp; | [_IfLetExpression_]  
> &nbsp;&nbsp; | [_MatchExpression_]  
> &nbsp;&nbsp; | [_ReturnExpression_]  

An expression may have two roles: it always produces a *value*, and it may have
*effects* (otherwise known as "side effects"). An expression *evaluates to* a
value, and has effects during *evaluation*. Many expressions contain
sub-expressions (operands). The meaning of each kind of expression dictates
several things:

* Whether or not to evaluate the sub-expressions when evaluating the expression
* The order in which to evaluate the sub-expressions
* How to combine the sub-expressions' values to obtain the value of the
  expression

In this way, the structure of expressions dictates the structure of execution.
Blocks are just another kind of expression, so blocks, statements, expressions,
and blocks again can recursively nest inside each other to an arbitrary depth.

## Expression precedence

The precedence of Rust operators and expressions is ordered as follows, going
from strong to weak. Binary Operators at the same precedence level are grouped
in the order given by their associativity.

| Operator/Expression         | Associativity       |
|-----------------------------|---------------------|
| Paths                       |                     |
| Method calls                |                     |
| Field expressions           | left to right       |
| Function calls, array indexing |                  |
| `?`                         |                     |
| Unary `-` `*` `!` `&` `&mut` |                    |
| `as`                        | left to right       |
| `*` `/` `%`                 | left to right       |
| `+` `-`                     | left to right       |
| `<<` `>>`                   | left to right       |
| `&`                         | left to right       |
| `^`                         | left to right       |
| <code>&#124;</code>         | left to right       |
| `==` `!=` `<` `>` `<=` `>=` | Require parentheses |
| `&&`                        | left to right       |
| <code>&#124;&#124;</code>   | left to right       |
| `..` `..=`                  | Require parentheses |
| `=` `+=` `-=` `*=` `/=` `%=` <br> `&=` <code>&#124;=</code> `^=` `<<=` `>>=` | right to left |
| `return` `break` closures   |                     |

## Place Expressions and Value Expressions

Expressions are divided into two main categories: place expressions and
value expressions. Likewise within each expression, sub-expressions may occur
in either place context or value context. The evaluation of an expression
depends both on its own category and the context it occurs within.

A *place expression* is an expression that represents a memory location. These
expressions are [paths] which refer to local variables, [static variables],
[dereferences]&nbsp;(`*expr`), [array indexing] expressions (`expr[expr]`),
[field] references (`expr.f`) and parenthesized place expressions. All other
expressions are value expressions.

A *value expression* is an expression that represents an actual value.

The left operand of an [assignment][assign] or [compound assignment] expression
is a place expression context, as is the single operand of a unary [borrow], and
the operand of any [implicit borrow]. The discriminant or subject of a
[match expression][match] and right side of a [let statement] is also a place
expression context. All other expression contexts are value expression contexts.

> Note: Historically, place expressions were called *lvalues* and value
> expressions were called *rvalues*.

### Moved and copied types

When a place expression is evaluated in a value expression context, or is bound
by value in a pattern, it denotes the value held _in_ that memory location. If
the type of that value implements [`Copy`], then the value will be copied. In
the remaining situations if that type is [`Sized`], then it may be possible to
move the value. Only the following place expressions may be moved out of:

* [Variables] which are not currently borrowed.
* [Temporary values](#temporary-lifetimes).
* [Fields][field] of a place expression which can be moved out of and
  doesn't implement [`Drop`].
* The result of [dereferencing] an expression with type [`Box<T>`] and that can
  also be moved out of.

Moving out of a place expression that evaluates to a local variable, the
location is deinitialized and cannot be read from again until it is
reinitialized. In all other cases, trying to use a place expression in a value
expression context is an error.

### Mutability

For a place expression to be [assigned][assign] to, mutably [borrowed][borrow],
[implicitly mutably borrowed], or bound to a pattern containing `ref mut` it
must be _mutable_. We call these *mutable place expressions*. In contrast,
other place expressions are called *immutable place expressions*.

The following expressions can be mutable place expression contexts:

* Mutable [variables], which are not currently borrowed.
* [Mutable `static` items].
* [Temporary values].
* [Fields][field], this evaluates the subexpression in a mutable place
  expression context.
* [Dereferences] of a `*mut T` pointer.
* Dereference of a variable, or field of a variable, with type `&mut T`. Note:
  This is an exception to the requirement of the next rule.
* Dereferences of a type that implements `DerefMut`, this then requires that
  the value being dereferenced is evaluated is a mutable place expression context.
* [Array indexing] of a type that implements `DerefMut`, this
  then evaluates the value being indexed, but not the index, in mutable place
  expression context.

### Temporary lifetimes

When using a value expression in most place expression contexts, a temporary
unnamed memory location is created initialized to that value and the expression
evaluates to that location instead, except if promoted to `'static`. Promotion
of a value expression to a `'static` slot occurs when the expression could be
written in a constant, borrowed, and dereferencing that borrow where the
expression was the originally written, without changing the runtime behavior.
That is, the promoted expression can be evaluated at compile-time and the
resulting value does not contain [interior mutability] or [destructors] (these
properties are determined based on the value where possible, e.g. `&None`
always has the type `&'static Option<_>`, as it contains nothing disallowed).
Otherwise, the lifetime of temporary values is typically

- the innermost enclosing statement; the tail expression of a block is
  considered part of the statement that encloses the block, or
- the condition expression or the loop conditional expression if the
  temporary is created in the condition expression of an `if` or in the loop
  conditional expression of a `while` expression.

When a temporary value expression is being created that is assigned into a
[`let` declaration][let], however, the temporary is created with the lifetime of
the enclosing block instead, as using the enclosing [`let` declaration][let]
would be a guaranteed error (since a pointer to the temporary
would be stored into a variable, but the temporary would be freed before the
variable could be used). The compiler uses simple syntactic rules to decide
which values are being assigned into a `let` binding, and therefore deserve a
longer temporary lifetime.

Here are some examples:

- `let x = foo(&temp())`. The expression `temp()` is a value expression. As it
  is being borrowed, a temporary is created which will be freed after
  the innermost enclosing statement; in this case, the `let` declaration.
- `let x = temp().foo()`. This is the same as the previous example,
  except that the value of `temp()` is being borrowed via autoref on a
  method-call. Here we are assuming that `foo()` is an `&self` method
  defined in some trait, say `Foo`. In other words, the expression
  `temp().foo()` is equivalent to `Foo::foo(&temp())`.
- `let x = if foo(&temp()) {bar()} else {baz()};`. The expression `temp()` is
  a value expression. As the temporary is created in the condition expression
  of an `if`, it will be freed at the end of the condition expression;
  in this example before the call to `bar` or `baz` is made.
- `let x = if temp().must_run_bar {bar()} else {baz()};`.
  Here we assume the type of `temp()` is a struct with a boolean field
  `must_run_bar`. As the previous example, the temporary corresponding to
  `temp()` will be freed at the end of the condition expression.
- `while foo(&temp()) {bar();}`. The temporary containing the return value from
  the call to `temp()` is created in the loop conditional expression. Hence it
  will be freed at the end of the loop conditional expression; in this example
  before the call to `bar` if the loop body is executed.
- `let x = &temp()`. Here, the same temporary is being assigned into
  `x`, rather than being passed as a parameter, and hence the
  temporary's lifetime is considered to be the enclosing block.
- `let x = SomeStruct { foo: &temp() }`. As in the previous case, the
  temporary is assigned into a struct which is then assigned into a
  binding, and hence it is given the lifetime of the enclosing block.
- `let x = [ &temp() ]`. As in the previous case, the
  temporary is assigned into an array which is then assigned into a
  binding, and hence it is given the lifetime of the enclosing block.
- `let ref x = temp()`. In this case, the temporary is created using a ref
  binding, but the result is the same: the lifetime is extended to the enclosing
  block.

### Implicit Borrows

Certain expressions will treat an expression as a place expression by implicitly
borrowing it. For example, it is possible to compare two unsized [slices] for
equality directly, because the `==` operator implicitly borrows it's operands:

```rust
# let c = [1, 2, 3];
# let d = vec![1, 2, 3];
let a: &[i32];
let b: &[i32];
# a = &c;
# b = &d;
// ...
*a == *b;
// Equivalent form:
::std::cmp::PartialEq::eq(&*a, &*b);
```

Implicit borrows may be taken in the following expressions:

* Left operand in [method-call] expressions.
* Left operand in [field] expressions.
* Left operand in [call expressions].
* Left operand in [array indexing] expressions.
* Operand of the [dereference operator] \(`*`).
* Operands of [comparison].
* Left operands of the [compound assignment].

## Constant expressions

Certain types of expressions can be evaluated at compile time. These are called
_constant expressions_. Certain places, such as in
[constants](items/constant-items.html) and [statics](items/static-items.html),
require a constant expression, and are always evaluated at compile time. In
other places, such as in [`let` statements](statements.html#let-statements),
constant expressions may be evaluated at compile time. If errors, such as out
of bounds [array indexing] or [overflow] occurs,
then it is a compiler error if the value must be evaluated at compile time,
otherwise it is just a warning, but the code will most likely panic when run.

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

## Overloading Traits

Many of the following operators and expressions can also be overloaded for
other types using traits in `std::ops` or `std::cmp`. These traits also
exist in `core::ops` and `core::cmp` with the same names.

[block expressions]:    expressions/block-expr.html
[call expressions]:     expressions/call-expr.html
[closure expressions]:  expressions/closure-expr.html
[enum variant]:         expressions/enum-variant-expr.html
[field]:                expressions/field-expr.html
[grouped]:              expressions/grouped-expr.html
[literals]:             expressions/literal-expr.html
[match]:                expressions/match-expr.html
[method-call]:          expressions/method-call-expr.html
[paths]:                expressions/path-expr.html
[range expressions]:    expressions/range-expr.html
[struct]:               expressions/struct-expr.html
[tuple expressions]:    expressions/tuple-expr.html

[array expressions]:    expressions/array-expr.html
[array indexing]:       expressions/array-expr.html#array-and-slice-indexing-expressions

[arithmetic, logical]:  expressions/operator-expr.html#arithmetic-and-logical-binary-operators
[assign]:               expressions/operator-expr.html#assignment-expressions
[borrow]:               expressions/operator-expr.html#borrow-operators
[cast]:                 expressions/operator-expr.html#type-cast-expressions
[comparison]:           expressions/operator-expr.html#comparison-operators
[compound assignment]:  expressions/operator-expr.html#compound-assignment-expressions
[dereferences]:         expressions/operator-expr.html#the-dereference-operator
[dereferencing]:        expressions/operator-expr.html#the-dereference-operator
[dereference operator]: expressions/operator-expr.html#the-dereference-operator
[lazy boolean]:         expressions/operator-expr.html#lazy-boolean-operators
[negation]:             expressions/operator-expr.html#negation-operators
[overflow]:             expressions/operator-expr.html#overflow

[destructors]:          destructors.html
[interior mutability]:  interior-mutability.html
[`Box<T>`]:             ../std/boxed/struct.Box.html
[`Copy`]:               special-types-and-traits.html#copy
[`Drop`]:               special-types-and-traits.html#drop
[`Sized`]:              special-types-and-traits.html#sized
[implicit borrow]:      #implicit-borrows
[implicitly mutably borrowed]: #implicit-borrows
[let]:                  statements.html#let-statements
[let statement]:        statements.html#let-statements
[Mutable `static` items]: items/static-items.html#mutable-statics
[slice]:                types.html#array-and-slice-types
[static variables]:     items/static-items.html
[Temporary values]:     #temporary-lifetimes
[Variables]:            variables.html

[_ArrayExpression_]:              expressions/array-expr.html
[_BlockExpression_]:              expressions/block-expr.html
[_BreakExpression_]:              expressions/loop-expr.html#break-expressions
[_CallExpression_]:               expressions/call-expr.html
[_ClosureExpression_]:            expressions/closure-expr.html
[_ContinueExpression_]:           expressions/loop-expr.html#continue-expressions
[_EnumerationVariantExpression_]: expressions/enum-variant-expr.html
[_FieldExpression_]:              expressions/field-expr.html
[_GroupedExpression_]:            expressions/grouped-expr.html
[_IfExpression_]:                 expressions/if-expr.html#if-expressions
[_IfLetExpression_]:              expressions/if-expr.html#if-let-expressions
[_IndexExpression_]:              expressions/array-expr.html#array-and-slice-indexing-expressions
[_LiteralExpression_]:            expressions/literal-expr.html
[_LoopExpression_]:               expressions/loop-expr.html
[_MatchExpression_]:              expressions/match-expr.html
[_MethodCallExpression_]:         expressions/method-call-expr.html
[_OperatorExpression_]:           expressions/operator-expr.html
[_PathExpression_]:               expressions/path-expr.html
[_RangeExpression_]:              expressions/range-expr.html
[_ReturnExpression_]:             expressions/return-expr.html
[_StructExpression_]:             expressions/struct-expr.html
[_TupleExpression_]:              expressions/tuple-expr.html
[_TupleIndexingExpression_]:      expressions/tuple-expr.html#tuple-indexing-expressions
