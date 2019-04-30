[![Build Status](https://api.travis-ci.org/GuillaumeGomez/macro-utils.png?branch=master)](https://travis-ci.org/GuillaumeGomez/macro-utils) [![Build status](https://ci.appveyor.com/api/projects/status/xnoltw6xvdyj70k6?svg=true)](https://ci.appveyor.com/project/GuillaumeGomez/macro-utils)

# macro-utils

Some macros to help writing better code or just having fun. 

## Usage

To use it in your project, just add the following lines:

```rust
#[macro_use]
extern crate macro_utils;
```

### Examples

```rust
#[macro_use]
extern crate macro_utils;

fn main() {
    let s = "bateau";

    if_match! {
        s == "voiture" => println!("It rolls!"),
        s == "avion"   => println!("It flies!"),
        s == "pieds"   => println!("It walks!"),
        s == "fusée"   => println!("It goes through space!"),
        s == "bateau"  => println!("It moves on water!"),
        else           => println!("I dont't know how it moves...")
    }

    let y = 4;
    let x = tern_c! { (y & 1 == 0) ? { "even" } : { "odd" } };
    let x = tern_python! { { "it's even" } if (y & 1 == 0) else { "it's odd" } };
    let x = tern_haskell! { if (y & 1 == 0) then { "it's even" } else { "it's odd" } };
}
```
