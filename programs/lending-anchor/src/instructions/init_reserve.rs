use crate::{
    account_data::{
        lending_market::*,
        reserve::{
            reserve_collateral::ReserveCollateral,
            reserve_config::ReserveConfig,
            reserve_liquidity::{NewReserveLiquidityParams, ReserveLiquidity},
            InitReserveParams, Reserve,
        },
    },
    errors::LendingError,
    math::common::WAD,
    pyth::{get_pyth_price, get_pyth_product_quote_currency},
    require_lt_100, require_lte_100,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{
    initialize_account, initialize_mint, mint_to, transfer, InitializeAccount, InitializeMint,
    Mint, MintTo, Token, TokenAccount, Transfer,
};
use pyth_sdk_solana::state::load_product_account;

/// Initializes a new lending market reserve.
#[derive(Accounts)]
pub struct InitReserve<'info> {
    /// Source liquidity token account
    /// $authority can transfer $liquidity_amount.
    #[account(mut)]
    pub source_liquidity: Account<'info, TokenAccount>,

    /// Destination collateral token account
    /// user's collateral token account
    #[account(init, payer = lending_market_owner, space = TokenAccount::LEN + 8)]
    pub destination_collateral: Account<'info, TokenAccount>,

    /// Reserve account
    #[account(init, payer = lending_market_owner, space = TokenAccount::LEN + 8)]
    pub reserve: Account<'info, Reserve>,

    /// Reserve liquidity SPL Token mint
    pub reserve_liquidity_mint: Account<'info, Mint>,

    /// Reserve liquidity supply SPL Token account
    #[account(init, payer = lending_market_owner, space = TokenAccount::LEN + 8)]
    pub reserve_liquidity_supply: Account<'info, TokenAccount>,

    /// Reserve liquidity fee receiver
    #[account(init, payer = lending_market_owner, space = TokenAccount::LEN + 8)]
    pub reserve_liquidity_fee_receiver: Account<'info, TokenAccount>,

    /// Reserve collateral SPL Token mint
    #[account(init, payer = lending_market_owner, space = TokenAccount::LEN + 8)]
    pub reserve_collateral_mint: Account<'info, Mint>,

    /// Reserve collateral token supply
    #[account(init, payer = lending_market_owner, space = TokenAccount::LEN + 8)]
    pub reserve_collateral_supply: Account<'info, TokenAccount>,

    /// CHECK: Pyth product account
    #[account(
        constraint = *pyth_product.owner == lending_market.oracle_program_id @ LendingError::InvalidOracleConfig,
    )]
    pub pyth_product: UncheckedAccount<'info>,

    /// CHECK: Pyth price account
    /// This will be used as the reserve liquidity oracle account
    #[account(
        constraint = *pyth_price.owner == lending_market.oracle_program_id @ LendingError::InvalidOracleConfig,
    )]
    pub pyth_price: UncheckedAccount<'info>,

    /// Lending market account
    pub lending_market: Account<'info, LendingMarket>,

    /// CHECK: Derived lending market authority
    #[account(
        seeds = [lending_market.key().as_ref()],
        bump = lending_market.bump_seed
    )]
    pub lending_market_authority: UncheckedAccount<'info>,

    /// Lending market owner
    #[account(
        mut,
        constraint = lending_market.owner == lending_market_owner.key() @ LendingError::InvalidMarketOwner,
    )]
    pub lending_market_owner: Signer<'info>,

    /// User transfer authority ($authority)
    pub user_transfer_authority: Signer<'info>,

    /// system program
    pub system_program: Program<'info, System>,

    /// token program
    pub token_program: Program<'info, Token>,

    /// rent sysvar
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitReserve<'info> {
    pub fn into_initialize_liquidity_supply_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeAccount<'info>> {
        let cpi_accounts = InitializeAccount {
            account: self.reserve_liquidity_supply.to_account_info(),
            mint: self.reserve_liquidity_mint.to_account_info(),
            authority: self.lending_market_authority.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    pub fn into_initialize_liqudity_fee_receiver_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeAccount<'info>> {
        let cpi_accounts = InitializeAccount {
            account: self.reserve_liquidity_fee_receiver.to_account_info(),
            mint: self.reserve_liquidity_mint.to_account_info(),
            authority: self.lending_market_authority.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    pub fn into_initialize_collateral_mint_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeMint<'info>> {
        let cpi_accounts = InitializeMint {
            mint: self.reserve_collateral_mint.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    pub fn into_initialize_collateral_supply_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeAccount<'info>> {
        let cpi_accounts = InitializeAccount {
            account: self.reserve_collateral_supply.to_account_info(),
            mint: self.reserve_collateral_mint.to_account_info(),
            authority: self.lending_market_authority.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    pub fn into_initialize_destination_collateral_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeAccount<'info>> {
        let cpi_accounts = InitializeAccount {
            account: self.destination_collateral.to_account_info(),
            mint: self.reserve_collateral_mint.to_account_info(),
            authority: self.user_transfer_authority.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    pub fn into_transfer_user_liquidity_to_supply_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.source_liquidity.to_account_info(),
            to: self.reserve_liquidity_supply.to_account_info(),
            authority: self.user_transfer_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    pub fn into_mint_to_destination_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.reserve_collateral_mint.to_account_info(),
            to: self.destination_collateral.to_account_info(),
            authority: self.lending_market_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

pub fn process_init_reserve(
    ctx: Context<InitReserve>,
    liquidity_amount: u64,
    config: ReserveConfig,
) -> Result<()> {
    require_neq!(
        liquidity_amount,
        0_u64,
        LendingError::ReserveNotInitializedWithLiquidity
    );
    require_lt_100!(config.optimal_utilization_rate, LendingError::InvalidConfig);
    require_lt_100!(config.loan_to_value_ratio, LendingError::InvalidConfig);
    require_gt!(
        config.liquidation_threshold,
        config.loan_to_value_ratio,
        LendingError::InvalidConfig
    );
    require_lte_100!(config.liquidation_threshold, LendingError::InvalidConfig);
    require!(
        config.min_borrow_rate <= config.optimal_borrow_rate
            && config.optimal_borrow_rate <= config.max_borrow_rate,
        LendingError::InvalidConfig
    );
    require_gt!(WAD, config.fees.borrow_fee_wad, LendingError::InvalidConfig);
    require_gt!(
        WAD,
        config.fees.flash_loan_fee_wad,
        LendingError::InvalidConfig
    );
    require_keys_neq!(
        ctx.accounts.reserve_liquidity_supply.key(),
        ctx.accounts.source_liquidity.key(),
        LendingError::InvalidAccountInput
    );

    // pythの準備
    let pyth_product_data = &(*ctx.accounts.pyth_product.data).borrow();
    let pyth_product =
        load_product_account(&pyth_product_data).map_err(|_| ProgramError::InvalidAccountData)?;

    let pyth_price_pubkey = *ctx.accounts.pyth_price.key;

    require_keys_eq!(
        Pubkey::new_from_array(pyth_product.px_acc.val),
        pyth_price_pubkey,
        LendingError::InvalidOracleConfig
    );

    let quote_currency = get_pyth_product_quote_currency(pyth_product)?;
    if ctx.accounts.lending_market.quote_currency != quote_currency {
        return Err(LendingError::InvalidOracleConfig.into());
    }

    let market_price = get_pyth_price(&ctx.accounts.pyth_price.to_account_info())?;

    let lending_market_pubkey = ctx.accounts.lending_market.key();
    let authority_signer_seeds = &[
        lending_market_pubkey.as_ref(),
        &[ctx.accounts.lending_market.bump_seed],
    ];
    let lending_market_authority_pubkey =
        Pubkey::create_program_address(authority_signer_seeds, ctx.program_id).unwrap();
    require_keys_eq!(
        lending_market_authority_pubkey,
        *ctx.accounts.lending_market_authority.key,
        LendingError::InvalidMarketAuthority
    );

    ctx.accounts.reserve.init(InitReserveParams {
        current_slot: Clock::get()?.slot,
        lending_market: ctx.accounts.lending_market.key(),
        liquidity: ReserveLiquidity::new(NewReserveLiquidityParams {
            mint_pubkey: ctx.accounts.reserve_liquidity_mint.key(),
            mint_decimals: ctx.accounts.reserve_liquidity_mint.decimals,
            supply_pubkey: ctx.accounts.reserve_liquidity_supply.key(),
            fee_receiver: ctx.accounts.reserve_liquidity_fee_receiver.key(),
            oracle_pubkey: ctx.accounts.pyth_price.key(),
            market_price,
        }),
        collateral: ReserveCollateral::new(
            ctx.accounts.reserve_collateral_mint.key(),
            0,
            ctx.accounts.reserve_collateral_supply.key(),
        ),
        config,
    });

    // 実際にdepositとmintはしていないが，感覚としてはDB上（account上の変数）の値を変更させて同期させている
    // depositとmintは関数一番下の2つのcpiでやる
    let collateral_amount = ctx.accounts.reserve.deposit_liquidity(liquidity_amount)?;

    // liquidity supplyのaccountの初期化
    initialize_account(ctx.accounts.into_initialize_liquidity_supply_ctx())?;
    // liquidity fee receiverの初期化
    initialize_account(ctx.accounts.into_initialize_liqudity_fee_receiver_ctx())?;
    // collateral用のmint accountを作る
    initialize_mint(
        ctx.accounts.into_initialize_collateral_mint_ctx(),
        ctx.accounts.reserve_liquidity_mint.decimals,
        ctx.accounts.lending_market_authority.key,
        Some(ctx.accounts.lending_market_authority.key),
    )?;
    // collateral supplyのaccountの初期化
    initialize_account(ctx.accounts.into_initialize_collateral_supply_ctx())?;
    // userのcollateral tokenのaccountを作ってあげる
    initialize_account(ctx.accounts.into_initialize_destination_collateral_ctx())?;

    // userのliquidityをlending_marketが持っているliquidity_supplyに移動
    transfer(
        ctx.accounts.into_transfer_user_liquidity_to_supply_ctx(),
        liquidity_amount,
    )?;

    // 代わりにcollateralをmintしてあげる
    mint_to(
        ctx.accounts.into_mint_to_destination_ctx(),
        collateral_amount,
    )?;

    Ok(())
}
