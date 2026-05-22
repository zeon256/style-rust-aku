#[macro_export]
macro_rules! info {
    ($($tt:tt)*) => {};
}

#[macro_export]
macro_rules! warn {
    ($($tt:tt)*) => {};
}

#[macro_export]
macro_rules! error {
    ($($tt:tt)*) => {};
}

#[macro_export]
macro_rules! debug {
    ($($tt:tt)*) => {};
}

extern crate self as tracing;

fn main() {
    tracing::info!("hello");
    tracing::warn!(target: "app", "hello");
    ::tracing::error!("hello");

    use tracing::debug;
    debug!("already imported");
}
