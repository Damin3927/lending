import { connection, program, QUOTE_CURRENCY } from "../common";
import { airdropTo, readKeypairFile } from "./util";
import { PublicKey, Keypair, Transaction } from "@solana/web3.js";
import { web3 } from "@project-serum/anchor";
import { initLendingMarketIx } from "../instructions/init_lending_market";
import { IDL } from "../../target/types/lending_anchor";

export class TestLendingMarket {
  /**
   * @param keypair - The keypair of this account
   * @param owner - The owner of this account
   * @param authority - The owner authority which can add new reserves
   * @param oracleProgramId - The oracle program id (Pyth)
   */
  constructor(
    private readonly keypair: Keypair,
    public readonly owner: Keypair,
    public readonly authority: PublicKey,
    public readonly quoteCurrency: number[],
    public readonly oracleProgramId: PublicKey
  ) {}

  static async init() {
    const lendingMarketOwner = readKeypairFile("tests/fixtures/lending_market_owner.json");
    await airdropTo(lendingMarketOwner.publicKey, 1);
    const oracleProgramId = readKeypairFile("tests/fixtures/oracle_program_id.json").publicKey;

    const lendingMarketKeypair = Keypair.generate();
    const lendingMarketPubkey = lendingMarketKeypair.publicKey;
    const [lendingMarketAuthority, _bumpSeed] = PublicKey.findProgramAddressSync(
      [lendingMarketPubkey.toBuffer()],
      program.programId
    );

    return new TestLendingMarket(
      lendingMarketKeypair,
      lendingMarketOwner,
      lendingMarketAuthority,
      QUOTE_CURRENCY,
      oracleProgramId
    );
  }

  async createLendingMarket() {
    const transaction = new Transaction().add(
      await initLendingMarketIx(this.owner.publicKey, this.quoteCurrency, this.keypair.publicKey, this.oracleProgramId)
    );
    transaction.feePayer = this.owner.publicKey;

    await web3.sendAndConfirmTransaction(connection, transaction, [this.owner, this.keypair]);
    return this;
  }

  async getState() {
    return await program.account.lendingMarket.fetch(this.keypair.publicKey);
  }

  async validateState() {
    const lendingMarket = await this.getState();
    expect(lendingMarket.version).toBe(Number(IDL.constants[0].value));
    expect(lendingMarket.owner).toEqual(this.owner.publicKey);
    expect(lendingMarket.quoteCurrency).toEqual(this.quoteCurrency);
  }
}
