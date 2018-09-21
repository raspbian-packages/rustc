# The on-demand SLG solver

## Description of how it works

The basis of the solver is the `Forest` type. A *forest* stores a
collection of *tables* as well as a *stack*. Each *table* represents
the stored results of a particular query that is being performed, as
well as the various *strands*, which are basically suspended
computations that may be used to find more answers. Tables are
interdependent: solving one query may require solving others.

### Walkthrough

Perhaps the easiest way to explain how the solver works is to walk
through an example. Let's imagine that we have the following program:

```rust
trait Debug { }

struct u32 { }
impl Debug for u32 { }

struct Rc<T> { }
impl<T: Debug> Debug for Rc<T> { }

struct Vec<T> { }
impl<T: Debug> Debug for Vec<T> { }
```

Now imagine that we want to find answers for the query `exists<T> {
Rc<T>: Debug }`. The first step would be to u-canonicalize this query; this
is the act of giving canonical names to all the unbound inference variables based on the 
order of their left-most appearance, as well as canonicalizing the universes of any
universally bound names (e.g., the `T` in `forall<T> { ... }`). In this case, there are no
universally bound names, but the canonical form Q of the query might look something like:

    Rc<?0>: Debug
    
where `?0` is a variable in the root universe U0. We would then go and
look for a table with this as the key: since the forest is empty, this
lookup will fail, and we will create a new table T0, corresponding to
the u-canonical goal Q.

**Ignoring negative reasoning and regions.** To start, we'll ignore
the possibility of negative goals like `not { Foo }`. We'll phase them
in later, as they bring several complications.

**Creating a table.** When we first create a table, we also initialize
it with a set of *initial strands*. A "strand" is kind of like a
"thread" for the solver: it contains a particular way to produce an
answer. The initial set of strands for a goal like `Rc<?0>: Debug`
(i.e., a "domain goal") is determined by looking for *clauses* in the
environment. In Rust, these clauses derive from impls, but also from
where-clauses that are in scope. In the case of our example, there
would be three clauses, each coming from the program. Using a
Prolog-like notation, these look like:

```
(u32: Debug).
(Rc<T>: Debug) :- (T: Debug).
(Vec<T>: Debug) :- (T: Debug).
```

To create our initial strands, then, we will try to apply each of
these clauses to our goal of `Rc<?0>: Debug`. The first and third
clauses are inapplicable because `u32` and `Vec<?0>` cannot be unified
with `Rc<?0>`. The second clause, however, will work.

**What is a strand?** Let's talk a bit more about what a strand *is*. In the code, a strand
is the combination of an inference table, an X-clause, and (possibly)
a selected subgoal from that X-clause. But what is an X-clause
(`ExClause`, in the code)? An X-clause pulls together a few things:

- The current state of the goal we are trying to prove;
- A set of subgoals that have yet to be proven;
- There are also a few things we're ignoring for now:
  - delayed literals, region constraints

The general form of an X-clause is written much like a Prolog clause,
but with somewhat different semantics. Since we're ignoring delayed
literals and region constraints, an X-clause just looks like this:

    G :- L
    
where G is a goal and L is a set of subgoals that must be proven.
(The L stands for *literal* -- when we address negative reasoning, a
literal will be either a positive or negative subgoal.) The idea is
that if we are able to prove L then the goal G can be considered true.

In the case of our example, we would wind up creating one strand, with
an X-clause like so:

    (Rc<?T>: Debug) :- (?T: Debug)

Here, the `?T` refers to one of the inference variables created in the
inference table that accompanies the strand. (I'll use named variables
to refer to inference variables, and numbered variables like `?0` to
refer to variables in a canonicalized goal; in the code, however, they
are both represented with an index.)

For each strand, we also optionally store a *selected subgoal*. This
is the subgoal after the turnstile (`:-`) that we are currently trying
to prove in this strand. Initally, when a strand is first created,
there is no selected subgoal.

**Activating a strand.** Now that we have created the table T0 and
initialized it with strands, we have to actually try and produce an
answer. We do this by invoking the `ensure_answer` operation on the
table: specifically, we say `ensure_answer(T0, A0)`, meaning "ensure
that there is a 0th answer".

Remember that tables store not only strands, but also a vector of
cached answers. The first thing that `ensure_answer` does is to check
whether answer 0 is in this vector. If so, we can just return
immediately.  In this case, the vector will be empty, and hence that
does not apply (this becomes important for cyclic checks later on).

When there is no cached answer, `ensure_answer` will try to produce
one.  It does this by selecting a strand from the set of active
strands -- the strands are stored in a `VecDeque` and hence processed
in a round-robin fashion. Right now, we have only one strand, storing
the following X-clause with no selected subgoal:

    (Rc<?T>: Debug) :- (?T: Debug)

When we activate the strand, we see that we have no selected subgoal,
and so we first pick one of the subgoals to process. Here, there is only
one (`?T: Debug`), so that becomes the selected subgoal, changing
the state of the strand to:

    (Rc<?T>: Debug) :- selected(?T: Debug, A0)
    
Here, we write `selected(L, An)` to indicate that (a) the literal `L`
is the selected subgoal and (b) which answer `An` we are looking for. We
start out looking for `A0`.

**Processing the selected subgoal.** Next, we have to try and find an
answer to this selected goal. To do that, we will u-canonicalize it
and try to find an associated table. In this case, the u-canonical
form of the subgoal is `?0: Debug`: we don't have a table yet for
that, so we can create a new one, T1. As before, we'll initialize T1
with strands. In this case, there will be three strands, because all
the program clauses are potentially applicable. Those three strands
will be:

- `(u32: Debug) :-`, derived from the program clause `(u32: Debug).`.
  - Note: This strand has no subgoals.
- `(Vec<?U>: Debug) :- (?U: Debug)`, derived from the `Vec` impl.
- `(Rc<?U>: Debug) :- (?U: Debug)`, derived from the `Rc` impl.

We can thus summarize the state of the whole forest at this point as
follows:

```
Table T0 [Rc<?0>: Debug]
  Strands:
    (Rc<?T>: Debug) :- selected(?T: Debug, A0)
  
Table T1 [?0: Debug]
  Strands:
    (u32: Debug) :-
    (Vec<?U>: Debug) :- (?U: Debug)
    (Rc<?V>: Debug) :- (?V: Debug)
```
    
**Delegation between tables.** Now that the active strand from T0 has
created the table T1, it can try to extract an answer. It does this
via that same `ensure_answer` operation we saw before. In this case,
the strand would invoke `ensure_answer(T1, A0)`, since we will start
with the first answer. This will cause T1 to activate its first
strand, `u32: Debug :-`.

This strand is somewhat special: it has no subgoals at all. This means
that the goal is proven. We can therefore add `u32: Debug` to the set
of *answers* for our table, calling it answer A0 (it is the first
answer). The strand is then removed from the list of strands.

The state of table T1 is therefore:

```
Table T1 [?0: Debug]
  Answers:
    A0 = [?0 = u32]
  Strand:
    (Vec<?U>: Debug) :- (?U: Debug)
    (Rc<?V>: Debug) :- (?V: Debug)
```

Note that I am writing out the answer A0 as a substitution that can be
applied to the table goal; actually, in the code, the goals for each
X-clause are also represented as substitutions, but in this exposition
I've chosen to write them as full goals, following NFTD.
   
Since we now have an answer, `ensure_answer(T1, A0)` will return `Ok`
to the table T0, indicating that answer A0 is available. T0 now has
the job of incorporating that result into its active strand. It does
this in two ways. First, it creates a new strand that is looking for
the next possible answer of T1. Next, it incorpoates the answer from
A0 and removes the subgoal. The resulting state of table T0 is:

```
Table T0 [Rc<?0>: Debug]
  Strands:
    (Rc<?T>: Debug) :- selected(?T: Debug, A1)
    (Rc<u32>: Debug) :-
```

We then immediately activate the strand that incorporated the answer
(the `Rc<u32>: Debug` one). In this case, that strand has no further
subgoals, so it becomes an answer to the table T0. This answer can
then be returned up to our caller, and the whole forest goes quiescent
at this point (remember, we only do enough work to generate *one*
answer). The ending state of the forest at this point will be:

```
Table T0 [Rc<?0>: Debug]
  Answer:
    A0 = [?0 = u32]
  Strands:
    (Rc<?T>: Debug) :- selected(?T: Debug, A1)

Table T1 [?0: Debug]
  Answers:
    A0 = [?0 = u32]
  Strand:
    (Vec<?U>: Debug) :- (?U: Debug)
    (Rc<?V>: Debug) :- (?V: Debug)
```

Here you can see how the forest captures both the answers we have
created thus far *and* the strands that will let us try to produce
more answers later on.

## Heritage and acroynms

This solver implements the SLG solving technique, though extended to
accommodate hereditary harrop (HH) predicates, as well as the needs of
lazy normalization. 

Its design is kind of a fusion of [MiniKanren] and the following
papers, which I will refer to as EWFS and NTFD respectively:

> Efficient Top-Down Computation of Queries Under the Well-formed Semantics
> (Chen, Swift, and Warren; Journal of Logic Programming '95)

> A New Formulation of Tabled resolution With Delay
> (Swift; EPIA '99)

[MiniKanren]: http://minikanren.org/

In addition, I incorporated extensions from the following papers,
which I will refer to as SA and RR respectively, that describes how to
do introduce approximation when processing subgoals and so forth:

> Terminating Evaluation of Logic Programs with Finite Three-Valued Models
> Riguzzi and Swift; ACM Transactions on Computational Logic 2013
> (Introduces "subgoal abstraction", hence the name SA)
>
> Radial Restraint
> Grosof and Swift; 2013

Another useful paper that gives a kind of high-level overview of
concepts at play is the following:

> XSB: Extending Prolog with Tabled Logic Programming
> (Swift and Warren; Theory and Practice of Logic Programming '10)

There are a places where I intentionally diverged from the semantics
as described in the papers -- e.g. by more aggressively approximating
-- which I marked them with a comment DIVERGENCE. Those places may
want to be evaluated in the future.

A few other acronyms that I use:

- WAM: Warren abstract machine, an efficient way to evaluate Prolog programs.
  See <http://wambook.sourceforge.net/>.
- HH: Hereditary harrop predicates. What Chalk deals in.
  Popularized by Lambda Prolog.


