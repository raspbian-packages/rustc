// This file was auto-generated using 'src/etc/generate-deriving-span-tests.py'

#[derive(PartialEq)]
struct Error;

#[derive(Eq,PartialEq)]
enum Enum {
   A {
     x: Error //~ ERROR
   }
}

fn main() {}
