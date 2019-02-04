# About the compiler team

rustc is maintained by the
[Rust compiler team](https://www.rust-lang.org/en-US/team.html). The
people who belong to this team collectively work to track regressions
and implement new features. Members of the Rust compiler team are
people who have made significant contributions to rustc and its
design.

## Discussion

Currently the compiler team chats in a number of places. There is an
ongoing [thread] on the internals board about trying to find a permanent
home. In any case, you can find people in one of three places at the moment:

- The `#rustc` channel on mozilla's IRC (`irc.mozilla.org`)
- The `t-compiler` stream on [the Zulip instance](https://rust-lang.zulipchat.com/#narrow/stream/131828-t-compiler)
- The `compiler` channel on the [rust-lang discord](https://discord.gg/rust-lang)

## Rust compiler meeting

The compiler team has a weekly meeting where we do triage and try to
generally stay on top of new bugs, regressions, and other things. This
general plan for this meeting can be found in
[the rust-compiler-meeting etherpad][etherpad]. It works roughly as
follows:

- **Review P-high bugs:** P-high bugs are those that are sufficiently
  important for us to actively track progress. P-high bugs should
  ideally always have an assignee.
- **Look over new regressions:** we then look for new cases where the
  compiler broke previously working code in the wild. Regressions are
  almost always marked as P-high; the major exception would be bug
  fixes (though even there we often
  [aim to give warnings first][procedure]).
- **Check I-nominated issues:** These are issues where feedback from
  the team is desired.
- **Check for beta nominations:** These are nominations of things to
  backport to beta.

The meeting currently takes place on Thursdays at 10am Boston time
(UTC-4 typically, but daylight savings time sometimes makes things
complicated).

The meeting is held over a "chat medium" — it used to be IRC, but we
are currently in the process of evaluating other alternatives. Check
the [etherpad] to find the current home (and see
[this internals thread][thread] for some ongoing discussion).

[etherpad]: https://public.etherpad-mozilla.org/p/rust-compiler-meeting
[thread]: https://internals.rust-lang.org/t/where-should-the-compiler-team-and-perhaps-working-groups-chat/7894
[procedure]: https://forge.rust-lang.org/rustc-bug-fix-procedure.html

## Team membership

Membership in the Rust team is typically offered when someone has been
making significant contributions to the compiler for some
time. Membership is both a recognition but also an obligation:
compiler team members are generally expected to help with upkeep as
well as doing reviews and other work.

If you are interested in becoming a compiler team member, the first
thing to do is to start fixing some bugs, or get involved in a working
group. One good way to find bugs is to look for
[open issues tagged with E-easy](https://github.com/rust-lang/rust/issues?q=is%3Aopen+is%3Aissue+label%3AE-easy)
or
[E-mentor](https://github.com/rust-lang/rust/issues?q=is%3Aopen+is%3Aissue+label%3AE-mentor).

### r+ rights

Once you have made a number of individual PRs to rustc, we will often
offer r+ privileges. This means that you have the right to instruct
"bors" (the robot that manages which PRs get landed into rustc) to
merge a PR
([here are some instructions for how to talk to bors][homu-guide]).

[homu-guide]: https://buildbot2.rust-lang.org/homu/

The guidelines for reviewers are as follows:

- You are always welcome to review any PR, regardless of who it is
  assigned to.  However, do not r+ PRs unless:
  - You are confident in that part of the code.
  - You are confident that nobody else wants to review it first.
    - For example, sometimes people will express a desire to review a
      PR before it lands, perhaps because it touches a particularly
      sensitive part of the code.
- Always be polite when reviewing: you are a representative of the
  Rust project, so it is expected that you will go above and beyond
  when it comes to the [Code of Conduct].

[Code of Conduct]: https://www.rust-lang.org/en-US/conduct.html

### high-five

Once you have r+ rights, you can also be added to the high-five
rotation. high-five is the bot that assigns incoming PRs to
reviewers. If you are added, you will be randomly selected to review
PRs. If you find you are assigned a PR that you don't feel comfortable
reviewing, you can also leave a comment like `r? @so-and-so` to assign
to someone else — if you don't know who to request, just write `r?
@nikomatsakis for reassignment` and @nikomatsakis will pick someone
for you.

[hi5]: https://github.com/rust-highfive

Getting on the high-five list is much appreciated as it lowers the
review burden for all of us! However, if you don't have time to give
people timely feedback on their PRs, it may be better that you don't
get on the list.

### Full team membership

Full team membership is typically extended once someone made many
contributions to the Rust compiler over time, ideally (but not
necessarily) to multiple areas. Sometimes this might be implementing a
new feature, but it is also important — perhaps more important! — to
have time and willingness to help out with general upkeep such as
bugfixes, tracking regressions, and other less glamorous work.
