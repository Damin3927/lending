import { TestLendingMarket } from "./test_lending_market";
import { TestOracle } from "./test_oracle";
import { connection, program } from "../common";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  approve,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAccount,
  getAccount,
  getMint,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { BN } from "@project-serum/anchor";
import { constantOf } from "./util";

export class TestReserve {
  constructor(
    public readonly name: string,
    public readonly pubkey: PublicKey,
    public readonly lendingMarketPubkey: PublicKey,
    public readonly config: ReserveConfig,
    public readonly liquidityMintPubkey: PublicKey,
    public readonly liquidityMintDecimals: number,
    public readonly liquiditySupplyPubkey: PublicKey,
    public readonly liquidityFeeReceiverPubkey: PublicKey,
    public readonly liquidityHostPubkey: PublicKey,
    public readonly liquidityOraclePubkey: PublicKey,
    public readonly collateralMintPubkey: PublicKey,
    public readonly collateralSupplyPubkey: PublicKey,
    public readonly userLiquidityPubkey: PublicKey,
    public readonly userCollateralPubkey: PublicKey,
    public readonly marketPrice: BN
  ) {}

  static async init(
    name: string,
    lendingMarket: TestLendingMarket,
    oracle: TestOracle,
    liquidityAmount: BN,
    config: ReserveConfig,
    liquidityMintPubkey: PublicKey,
    userLiquidityPubkey: PublicKey,
    payer: Keypair,
    userAccountsOwner: Keypair
  ) {
    const reserveKeypair = Keypair.generate();
    const collateralMintKeypair = Keypair.generate();
    const collateralSupplyKeypair = Keypair.generate();
    const liquiditySupplyKeypair = Keypair.generate();
    const liquidityFeeReceiverKeypair = Keypair.generate();
    const liquidityHostKeypair = Keypair.generate();
    const userCollateralTokenKeypair = Keypair.generate();
    const userTransferAuthorityKeypair = Keypair.generate();

    const liquidityMint = await getMint(connection, liquidityMintPubkey);

    // send approve tx
    await approve(
      connection,
      payer,
      userLiquidityPubkey,
      userTransferAuthorityKeypair.publicKey,
      userAccountsOwner,
      liquidityAmount
    );

    // create liquidity host account
    await createAccount(
      connection,
      payer,
      liquidityMintPubkey,
      userAccountsOwner.publicKey,
      liquidityHostKeypair
    );

    const signers = [
      // payer,
      reserveKeypair,
      liquiditySupplyKeypair,
      liquidityFeeReceiverKeypair,
      collateralMintKeypair,
      collateralSupplyKeypair,
      lendingMarket.owner,
      userTransferAuthorityKeypair,
    ];
    const accounts = {
      sourceLiquidity: userLiquidityPubkey,
      destinationCollateral: userCollateralTokenKeypair.publicKey,
      reserve: reserveKeypair.publicKey,
      reserveLiquidityMint: liquidityMintPubkey,
      reserveLiquiditySupply: liquiditySupplyKeypair.publicKey,
      reserveLiquidityFeeReceiver: liquidityFeeReceiverKeypair.publicKey,
      reserveCollateralMint: collateralMintKeypair.publicKey,
      reserveCollateralSupply: collateralSupplyKeypair.publicKey,
      pythProduct: oracle.productPubkey,
      pythPrice: oracle.pricePubkey,
      lendingMarket: lendingMarket.keypair.publicKey,
      lendingMarketAuthority: lendingMarket.authority,
      lendingMarketOwner: lendingMarket.owner.publicKey,
      userTransferAuthority: userTransferAuthorityKeypair.publicKey,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
    };

    // init reserve
    await program.methods
      // @ts-ignore: type completion bug
      .initReserve(liquidityAmount, TEST_RESERVE_CONFIG)
      .accounts(accounts)
      .signers(signers)
      .rpc();

    return new TestReserve(
      name,
      reserveKeypair.publicKey,
      lendingMarket.keypair.publicKey,
      config,
      liquidityMintPubkey,
      liquidityMint.decimals,
      liquiditySupplyKeypair.publicKey,
      liquidityFeeReceiverKeypair.publicKey,
      liquidityHostKeypair.publicKey,
      oracle.pricePubkey,
      collateralMintKeypair.publicKey,
      collateralSupplyKeypair.publicKey,
      userLiquidityPubkey,
      userCollateralTokenKeypair.publicKey,
      oracle.price
    );
  }

  public async validateState() {
    const reserve = await program.account.reserve.fetch(this.pubkey);
    expect(reserve.lastUpdate.slot).toBeGreaterThan(0);
    expect(reserve.version).toBe(constantOf("PROGRAM_VERSION"));
    expect(reserve.lendingMarket).toEqual(this.lendingMarketPubkey);
    expect(reserve.liquidity.mintPubkey).toEqual(this.liquidityMintPubkey);
    expect(reserve.liquidity.supplyPubkey).toEqual(this.liquiditySupplyPubkey);
    expect(reserve.collateral.mintPubkey).toEqual(this.collateralMintPubkey);
    expect(reserve.collateral.supplyPubkey).toEqual(
      this.collateralSupplyPubkey
    );
    expect(reserve.config).toEqual(this.config);
    expect(reserve.liquidity.oraclePubkey).toEqual(this.liquidityOraclePubkey);
    expect(reserve.liquidity.cumulativeBorrowRateWads).toBe(new BN(1));
    expect(reserve.liquidity.borrowedAmountWads).toBe(new BN(0));
    expect(reserve.liquidity.availableAmount).toBeGreaterThan(0);
    expect(reserve.collateral.mintTotalSupply).toBeGreaterThan(0);
  }

  public getLiquiditySupplyAccount() {
    return getAccount(connection, this.liquiditySupplyPubkey);
  }

  public getUserLiquidityAccount() {
    return getAccount(connection, this.userLiquidityPubkey);
  }

  public getUserCollateralAccount() {
    return getAccount(connection, this.userCollateralPubkey);
  }
}

export interface ReserveConfig {
  optimalUtilizationRate: number;
  loanToValueRatio: number;
  liquidationBonus: number;
  liquidationThreshold: number;
  minBorrowRate: number;
  optimalBorrowRate: number;
  maxBorrowRate: number;
  fees: ReserveFees;
}

export interface ReserveFees {
  borrowFeeWad: BN;
  flashLoanFeeWad: BN;
  hostFeePercentage: number;
}

export const TEST_RESERVE_CONFIG: ReserveConfig = {
  optimalUtilizationRate: 80,
  loanToValueRatio: 50,
  liquidationBonus: 5,
  liquidationThreshold: 55,
  minBorrowRate: 0,
  optimalBorrowRate: 4,
  maxBorrowRate: 30,
  fees: {
    borrowFeeWad: 100_000_000_000,
    flashLoanFeeWad: 3_000_000_000_000_000,
    hostFeePercentage: 20,
  },
};
