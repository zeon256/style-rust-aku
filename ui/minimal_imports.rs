fn main() {
    let _a: std::io::Result<()> = Ok(());
    let _b: std::result::Result<(), ()> = Ok(());
    let _c: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let _d: std::fmt::Error = std::fmt::Error;

    use std::io;
    // This should NOT trigger (only 2 segments)
    let _e: io::Result<()> = Ok(());

    // This should NOT trigger (underscore-prefixed macro helper path)
    let _f = _serde::de::SeqAccess::next_element();
}

mod _serde {
    pub mod de {
        pub struct SeqAccess;

        impl SeqAccess {
            pub fn next_element() {}
        }
    }
}
