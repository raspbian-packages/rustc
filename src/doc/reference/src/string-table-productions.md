# String table productions

Some rules in the grammar &mdash; notably [unary
operators], [binary operators], and [keywords][keywords] &mdash; are
given in a simplified form: as a listing of a table of unquoted, printable
whitespace-separated strings. These cases form a subset of the rules regarding
the [token][tokens] rule, and are assumed to be the result of a
lexical-analysis phase feeding the parser, driven by a DFA, operating over the
disjunction of all such string table entries.

When such a string enclosed in double-quotes (`"`) occurs inside the grammar,
it is an implicit reference to a single member of such a string table
production. See [tokens] for more information.

[binary operators]: expressions/operator-expr.html#arithmetic-and-logical-binary-operators
[keywords]: keywords.html
[tokens]: tokens.html
[unary operators]: expressions/operator-expr.html#borrow-operators
