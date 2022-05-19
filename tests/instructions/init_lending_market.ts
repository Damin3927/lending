import { program } from "../common";
import { SystemProgram, PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { TestLendingMarket } from "../helpers/test_lending_market";

describe("init_lending_market", () => {
  let lendingMarket: TestLendingMarket;

  beforeAll(async () => {
    lendingMarket = await (await TestLendingMarket.init()).createLendingMarket();
  });

  describe("proper initialization", () => {
    it("succeeds", async () => {
      await lendingMarket.validateState();
    });
  });

  describe("when the account has already initialized", () => {
    it("raises an error", async () => {
      try {
        await lendingMarket.createLendingMarket();
        fail();
      } catch {}
    });
  });
});

export const initLendingMarketIx = async (
  owner: PublicKey,
  quoteCurrency: number[],
  lendingMarketPubkey: PublicKey,
  oracleProgramId: PublicKey
) =>
  await program.methods
    .initLendingMarket(quoteCurrency)
    .accounts({
      owner,
      lendingMarket: lendingMarketPubkey,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      oracle: oracleProgramId,
    })
    .instruction();
