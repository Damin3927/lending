import { BN } from "@project-serum/anchor";
import { createAccount, NATIVE_MINT } from "@solana/spl-token";
import { Keypair } from "@solana/web3.js";
import { getPayer } from "../common";
import { TestLendingMarket } from "../helpers/test_lending_market";
import { TestOracle } from "../helpers/test_oracle";
import { TestReserve, TEST_RESERVE_CONFIG } from "../helpers/test_reserve";
import { constantOf, createAndMintToTokenAccount } from "../helpers/util";

describe("init_reserve", () => {
  let lendingMarket: TestLendingMarket;
  let reserve: TestReserve;
  let oracle: TestOracle;
  let userAccountsOwner: Keypair;

  beforeEach(async () => {
    lendingMarket = await (await TestLendingMarket.init()).createLendingMarket();
    userAccountsOwner = Keypair.generate();
    oracle = TestOracle.addSolOracle();
  });

  describe("proper initialization", () => {
    it("succeeds", async () => {
      const reserveAmount = new BN(42);

      const solUserLiquidityAccount = await createAndMintToTokenAccount(
        reserveAmount,
        userAccountsOwner.publicKey,
        await getPayer(),
        NATIVE_MINT
      );

      reserve = await TestReserve.init(
        "sol",
        lendingMarket,
        oracle,
        reserveAmount,
        TEST_RESERVE_CONFIG,
        NATIVE_MINT, // WSOL
        solUserLiquidityAccount,
        await getPayer(),
        userAccountsOwner
      );

      await reserve.validateState();

      const solLiquiditySupply = (await reserve.getLiquiditySupplyAccount()).amount;
      expect(solLiquiditySupply).toBe(reserveAmount);

      const userSolBalance = (await reserve.getUserLiquidityAccount()).amount;
      expect(userSolBalance).toBe(0);

      const userSolCollateralBalance = (await reserve.getUserCollateralAccount()).amount;
      expect(userSolCollateralBalance).toBe(reserveAmount * constantOf("INITIAL_COLLATERAL_RATIO"));
    });
  });
});
