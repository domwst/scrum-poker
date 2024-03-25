#[macro_export]
macro_rules! if_backend {
    {$($tokens:tt)*} => {
        cfg_if::cfg_if! {
            if #[cfg(feature = "ssr")] {
                $($tokens)*
            }
        }
    };
}

#[macro_export]
macro_rules! if_frontend {
    {$($tokens:tt)*} => {
        cfg_if::cfg_if! {
            if #[cfg(any(feature = "hydrate", feature = "csr"))] {
                $($tokens)*
            }
        }
    };
}
