/// Assert that a condition holds, with detailed error message
#[macro_export]
macro_rules! assert_with_context {
    ($cond:expr, $context:expr) => {
        if !$cond {
            panic!(
                "Assertion failed: {}\nContext: {}",
                stringify!($cond),
                $context
            );
        }
    };
}

/// Assert that an invariant holds
#[macro_export]
macro_rules! assert_invariant {
    ($cond:expr, $invariant_name:expr) => {
        if !$cond {
            panic!(
                "Invariant violated: {}\nCondition: {}",
                $invariant_name,
                stringify!($cond)
            );
        }
    };
}
