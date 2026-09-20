// Probe fixture: pre-expansion visibility of cfg-gated inline modules.
//
// Expected: NO_INLINE_MODULES fires on the first module (cfg-gated inline body)
// even in a non-test build; the production module stays silent.

#[cfg(test)]
mod tests {
    fn helper() {}
}

#[cfg(all(test, feature = "json"))]
mod other_tests {
    fn helper() {}
}

#[cfg(not(test))]
mod production {
    fn helper() {}
}

pub mod serde_string {
    pub fn serialize() {}
}

mod checks {
    #[test]
    fn catches() {}
}

#[cfg(test)]
mod outlined;

fn main() {}
