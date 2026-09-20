// compile-flags: --edition=2021
#![feature(register_tool)]
#![register_tool(mytest)]

fn main() {}

#[test]
fn t() {}

// Companion attribute must not cause a second warning.
#[should_panic]
#[test]
fn t2() {}

// Scoped (`path::to::test`) forms match on their terminal identifier.
struct S;
impl S {
    #[mytest::test]
    fn m(&self) {}
}

trait T {
    #[mytest::test]
    fn mt(&self) {}
}

#[mytest::test]
async fn t3() {}

// Valid: gated helper is not a test fn.
#[cfg(test)]
fn unit_helper() {}
