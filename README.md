##rustformat

This is a prototype for a rust formating tool.

###Building rustformat

~~~
cargo build
~~~

Note: Since rustformat relies on compiler internals (that are marked as unstable) you will need the nightly distribution of rust.

This will result in a rustformat executable.

###Usage

rustformat [path to one or more rust files]

This will replace the content of the files with its formated versions.

Please be aware of the fact that this is not meant for production use yet! The rustformat code is formated with rustformat, but besides that I cannot guarantee that semantic meaning is preserved.

Examples:
~~~
fn main() {
    let _immutable_binding
    = 1;
    let mut mutable_binding =1;

println!("Before mutation: {}", mutable_binding);

        // Ok
mutable_binding += 1

;

        println!("After mutation: {}", mutable_binding);
    // Error!
        _immutable_binding += 1;
    // FIXME ^ Comment out this line
}
~~~
becomes
~~~
fn main() {
    let _immutable_binding = 1;
    let mut mutable_binding = 1;

    println!("Before mutation: {}", mutable_binding);

    // Ok
    mutable_binding += 1;

    println!("After mutation: {}", mutable_binding);
    // Error!
    _immutable_binding += 1;
    // FIXME ^ Comment out this line
}
~~~
---
~~~
if n<0
{
    print!("{} is negative", n);
}
else
if n>0
{
    print!("{} is positive", n);
}
else
{
    print!("{} is zero", n);
}
~~~
becomes
~~~
if n < 0 {
    print!("{} is negative", n);
} else if n > 0 {
    print!("{} is positive", n);
} else {
    print!("{} is zero", n);
}
~~~
---
~~~
match format_file(filename.as_ref()) {
    Err(e) => { println!("{:?}", e); return; },
    Ok(_) => {}
}
~~~
becomes
~~~
match format_file(filename.as_ref()) {
    Err(e) => {
        println!("{:?}", e);
        return;
    },
    Ok(_) => {},
}
~~~

