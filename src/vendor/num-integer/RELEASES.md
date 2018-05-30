# Release 0.1.36

- [num-integer now has its own source repository][num-356] at [rust-num/num-integer][home].
- [Corrected the argument order documented in `Integer::is_multiple_of`][1]
- [There is now a `std` feature][5], enabled by default, along with the implication
  that building *without* this feature makes this a `#[no_std]` crate.
  - There is no difference in the API at this time.

**Contributors**: @cuviper, @jaystrictor

[home]: https://github.com/rust-num/num-integer
[num-356]: https://github.com/rust-num/num/pull/356
[1]: https://github.com/rust-num/num-integer/pull/1
[5]: https://github.com/rust-num/num-integer/pull/5


# Prior releases

No prior release notes were kept.  Thanks all the same to the many
contributors that have made this crate what it is!

