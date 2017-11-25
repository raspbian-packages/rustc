// Tests for the various helper functions used by the needless_continue
// lint that don't belong in utils.
extern crate clippy_lints;
use clippy_lints::needless_continue::{erode_from_back, erode_block, erode_from_front};

#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn test_erode_from_back() {
    let input = "\
{
    let x = 5;
    let y = format!(\"{}\", 42);
}";

    let expected = "\
{
    let x = 5;
    let y = format!(\"{}\", 42);";

    let got = erode_from_back(input);
    assert_eq!(expected, got);
}

#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn test_erode_from_back_no_brace() {
    let input = "\
let x = 5;
let y = something();
";
    let expected = "";
    let got = erode_from_back(input);
    assert_eq!(expected, got);
}

#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn test_erode_from_front() {
    let input = "
        {
            something();
            inside_a_block();
        }
    ";
    let expected =
"            something();
            inside_a_block();
        }
    ";
    let got = erode_from_front(input);
    println!("input: {}\nexpected:\n{}\ngot:\n{}", input, expected, got);
    assert_eq!(expected, got);
}

#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn test_erode_from_front_no_brace() {
    let input = "
            something();
            inside_a_block();
    ";
    let expected =
"something();
            inside_a_block();
    ";
    let got = erode_from_front(input);
    println!("input: {}\nexpected:\n{}\ngot:\n{}", input, expected, got);
    assert_eq!(expected, got);
}


#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn test_erode_block() {

    let input = "
        {
            something();
            inside_a_block();
        }
    ";
    let expected =
"            something();
            inside_a_block();";
    let got = erode_block(input);
    println!("input: {}\nexpected:\n{}\ngot:\n{}", input, expected, got);
    assert_eq!(expected, got);
}
