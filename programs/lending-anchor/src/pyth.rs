use crate::errors::LendingError;
use anchor_lang::prelude::*;
use pyth_sdk_solana::state::{
    load_price_account, PriceStatus, PriceType, ProductAccount, PROD_ATTR_SIZE,
};

pub fn get_pyth_product_quote_currency(pyth_product: &ProductAccount) -> Result<[u8; 32]> {
    const LEN: usize = 14;
    const KEY: &[u8; LEN] = b"quote_currency";

    let mut start = 0;
    while start < PROD_ATTR_SIZE {
        let mut length = pyth_product.attr[start] as usize;
        start += 1;

        if length == LEN {
            let mut end = start + length;
            require_gte!(PROD_ATTR_SIZE, end, LendingError::InvalidOracleConfig);

            let key = &pyth_product.attr[start..end];
            if key == KEY {
                start += length;
                length = pyth_product.attr[start] as usize;
                start += 1;

                end = start + length;
                require_gte!(32, length, LendingError::InvalidOracleConfig);
                require_gte!(PROD_ATTR_SIZE, end, LendingError::InvalidOracleConfig);

                let mut value = [0u8; 32];
                value[0..length].copy_from_slice(&pyth_product.attr[start..end]);
                return Ok(value);
            }
        }

        start += length;
        start += 1 + pyth_product.attr[start] as usize;
    }

    err!(LendingError::InvalidOracleConfig)
}

pub fn get_pyth_price(pyth_price_info: &AccountInfo) -> Result<u128> {
    const STALE_AFTER_SLOTS_ELAPSED: u64 = 5;

    let pyth_price_data = pyth_price_info.try_borrow_data()?;
    let pyth_price =
        load_price_account(&pyth_price_data).map_err(|_| ProgramError::InvalidAccountData)?;

    if pyth_price.ptype != PriceType::Price {
        msg!("Oracle price type is invalid");
        return Err(LendingError::InvalidOracleConfig.into());
    }

    if pyth_price.agg.status != PriceStatus::Trading {
        msg!("Oracle price status is invalid");
        return Err(LendingError::InvalidOracleConfig.into());
    }

    let slots_elapsed = Clock::get()?
        .slot
        .checked_sub(pyth_price.valid_slot)
        .ok_or(LendingError::MathOverflow)?;
    if slots_elapsed >= STALE_AFTER_SLOTS_ELAPSED {
        msg!("Oracle price is stale");
        return Err(LendingError::InvalidOracleConfig.into());
    }

    let price: u64 = pyth_price.agg.price.try_into().map_err(|_| {
        msg!("Oracle price cannot be negative");
        LendingError::InvalidOracleConfig
    })?;

    let market_price = if pyth_price.expo >= 0 {
        let exponent = pyth_price
            .expo
            .try_into()
            .map_err(|_| LendingError::MathOverflow)?;
        let zeros = 10u64
            .checked_pow(exponent)
            .ok_or(LendingError::MathOverflow)?;
        u128::from(price)
            .checked_mul(zeros as u128)
            .ok_or(LendingError::MathOverflow)?
    } else {
        let exponent = pyth_price
            .expo
            .checked_abs()
            .ok_or(LendingError::MathOverflow)?
            .try_into()
            .map_err(|_| LendingError::MathOverflow)?;
        let decimals = 10u64
            .checked_pow(exponent)
            .ok_or(LendingError::MathOverflow)?;
        u128::from(price)
            .checked_div(decimals as u128)
            .ok_or(LendingError::MathOverflow)?
    };

    Ok(market_price)
}
