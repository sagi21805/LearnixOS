use quote::quote;

#[unsafe(no_mangle)]
pub fn square(num: i32) -> i32 {
    num * num
}

macro_rules! square {
    ($num:expr) => {
        $num * $num
    };
}

fn foo() -> u32 {
    let x = 42;
    square!(x)
}

fn foo_expanded() -> u32 {
    let x = 42;
    x * x
}

macro_rules! unwrap_or_break {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => break,
        }
    };
}

fn main() {
    let data = vec![Some(1), Some(2), None, Some(4)];

    for d in data {
        let val = unwrap_or_break!(d); // breaks the loop on None
        println!("{}", val);
    }

    println!("done");
}

fn unwrap_or_break<T>(e: Option<T>) -> T {
    match e {
        Some(v) => v,
        None => break, // ERROR: `break` outside of a loop
    }
}

/// Macros are defined using the `macro_rules!` macro,
/// followed by the name of the macro.
macro_rules! unwrap_or_break {
    // Each rule is defined with the "() => {}" syntax,
    // in the parentheses we provide the pattern to match,
    // which uses `Metavariables` to capture parts of the input.
    ($e:expr) => {
        // Then, we can write 'regular' Rust code inside the macro body,
        // which uses the metavariables to generate the expanded code.
        match $e {
            Some(v) => v,
            None => break,
        }
    };
}

#[derive(WithHelperAttr)]
struct Foo {
    #[helper]
    bar: (),
}

#[return_as_is]
struct Bar {
    foo: (),
}

#[return_as_is]
fn bar() {}

quote! {
    struct Foo {
       bar: ()
    }

    fn main() {

    }
}

#[bitfields]
struct MyFlags {
    #[flag(r)]
    a: B2,
    b: B5,
    #[flag(rwc(30))]
    c: B3,
    #[flag(flag_type = ProtectionLevel)]
    d: B2,
    #[flag(r, dont_shift)]
    e: B3,
    f: B1,
}

struct SimpleFlags {
    a: B2,
    b: B1,
}

struct SimpleFlagsType(u8);
