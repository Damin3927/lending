import { TestLendingMarket } from "../helpers/test_lending_market";
import { Keypair, PublicKey } from "@solana/web3.js";
import { program } from "../common";
import { generateWealthKeypair, signatureVerificationError } from "../helpers/util";
import { customErrorOf } from "../helpers/util";

describe("set_lending_market_owner", () => {
  let lendingMarket: TestLendingMarket;
  const newOwner = Keypair.generate();

  beforeEach(async () => {
    lendingMarket = await (await TestLendingMarket.init()).createLendingMarket();
  });

  describe("proper update of owner", () => {
    it("succeeds", async () => {
      await lendingMarket.setNewOwner(newOwner);
      await lendingMarket.validateState();
    });
  });

  describe("when the given current owner is wrong", () => {
    it("raises an error", () => {
      expect(
        async () =>
          await lendingMarket.setNewOwner(newOwner, {
            currentOwner: await generateWealthKeypair(),
          })
      ).rejects.toThrow(customErrorOf("InvalidMarketOwner"));
    });
  });

  describe("when the tx's signer is not the owner", () => {
    it("raises an error", () => {
      expect(
        async () =>
          await lendingMarket.setNewOwner(newOwner, {
            signer: await generateWealthKeypair(),
          })
      ).rejects.toThrow(signatureVerificationError);
    });
  });
});

export const setLendingMarketIx = async (currentOwner: Keypair, lendingMarket: PublicKey, newOwner: PublicKey) =>
  await program.methods
    .setLendingMarketOwner(newOwner)
    .accounts({
      lendingMarket: lendingMarket,
      owner: currentOwner.publicKey,
    })
    .instruction();
