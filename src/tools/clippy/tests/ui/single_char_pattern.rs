use std::collections::HashSet;

fn main() {
    let x = "foo";
    x.split("x");
    x.split("xx");
    x.split('x');

    let y = "x";
    x.split(y);
    // Not yet testing for multi-byte characters
    // Changing `r.len() == 1` to `r.chars().count() == 1` in `lint_single_char_pattern`
    // should have done this but produced an ICE
    //
    // We may not want to suggest changing these anyway
    // See: https://github.com/rust-lang-nursery/rust-clippy/issues/650#issuecomment-184328984
    x.split("ß");
    x.split("ℝ");
    x.split("💣");
    // Can't use this lint for unicode code points which don't fit in a char
    x.split("❤️");
    x.contains("x");
    x.starts_with("x");
    x.ends_with("x");
    x.find("x");
    x.rfind("x");
    x.rsplit("x");
    x.split_terminator("x");
    x.rsplit_terminator("x");
    x.splitn(0, "x");
    x.rsplitn(0, "x");
    x.matches("x");
    x.rmatches("x");
    x.match_indices("x");
    x.rmatch_indices("x");
    x.trim_left_matches("x");
    x.trim_right_matches("x");

    let h = HashSet::<String>::new();
    h.contains("X"); // should not warn
}
