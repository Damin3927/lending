#[macro_export]
macro_rules! require_lt_100 {
    ($value: expr, $error_code: expr $(,)?) => {
        if $value >= 100_u8 {
            return Err(error!($error_code));
        }
    };
    ($value: expr $(,)?) => {
        if $value >= 100 {
            return Err(error!(anchor_lang::error::ErrorCode::RequireGtViolated));
        }
    };
}

#[macro_export]
macro_rules! require_lte_100 {
    ($value: expr, $error_code: expr $(,)?) => {
        if $value > 100_u8 {
            return Err(error!($error_code));
        }
    };
    ($value: expr $(,)?) => {
        if $value > 100 {
            return Err(error!(anchor_lang::error::ErrorCode::RequireGtViolated));
        }
    };
}
