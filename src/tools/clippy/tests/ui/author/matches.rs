#![feature(custom_attribute)]

fn main() {
    #[clippy(author)]
    let a = match 42 {
        16 => 5,
        17 => {
            let x = 3;
            x
        },
        _ => 1,
    };
}
