use std::fmt::{Debug, Display};

pub fn show<T: Display>(name: &str, x: Option<T>) {
    let s = match x {
        Some(x) => x.to_string(),
        None => String::from("–"),
    };

    println!("{:>21} {s}", format!("{name}:"));
}

pub fn show_<T: Debug>(name: &str, x: Option<T>) {
    let s = match x {
        Some(x) => format!("{x:?}"),
        None => String::from("–"),
    };

    println!("{:>21} {s}", format!("{name}:"));
}
