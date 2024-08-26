pub mod types;

/// Util macro in under to check if all value are none
#[macro_export]
macro_rules! all_none {
    ($($maybe: expr),*) => {
        $($maybe.is_none())&&*
    };
}

/// Util macro in under to check if all value are some
#[macro_export]
macro_rules! all_some {
    ($($maybe: expr),*) => {
        $($maybe.is_some())&&*
    };
}
