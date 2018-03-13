#![allow(unused)]

use std::collections::HashSet;
use std::collections::VecDeque;
use std::rc::{self, Rc};
use std::sync::{self, Arc};

fn main() {}

fn clone_on_copy() {
    42.clone();

    vec![1].clone(); // ok, not a Copy type
    Some(vec![1]).clone(); // ok, not a Copy type
    (&42).clone();
}

fn clone_on_ref_ptr() {
    let rc = Rc::new(true);
    let arc = Arc::new(true);

    let rcweak = Rc::downgrade(&rc);
    let arc_weak = Arc::downgrade(&arc);

    rc.clone();
    Rc::clone(&rc);

    arc.clone();
    Arc::clone(&arc);

    rcweak.clone();
    rc::Weak::clone(&rcweak);

    arc_weak.clone();
    sync::Weak::clone(&arc_weak);


}

fn clone_on_copy_generic<T: Copy>(t: T) {
    t.clone();

    Some(t).clone();
}

fn clone_on_double_ref() {
    let x = vec![1];
    let y = &&x;
    let z: &Vec<_> = y.clone();

    println!("{:p} {:p}",*y, z);
}

fn iter_clone_collect() {
    let v = [1,2,3,4,5];
    let v2 : Vec<isize> = v.iter().cloned().collect();
    let v3 : HashSet<isize> = v.iter().cloned().collect();
    let v4 : VecDeque<isize> = v.iter().cloned().collect();
}
