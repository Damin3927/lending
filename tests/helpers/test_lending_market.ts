import { connection, program, QUOTE_CURRENCY } from "../common";
import { airdropTo, readKeypairFile } from "./util";
import { PublicKey, Keypair, Transaction } from "@solana/web3.js";
import { web3 } from "@project-serum/anchor";
import { initLendingMarketIx } from "../instructions/init_lending_market";
import { IDL } from "../../target/types/lending_anchor";

export class TestLendingMarket {
  constructor(
    public pubkey: PublicKey,
    public owner: Keypair,
    public authority: PublicKey,
    public quoteCurrency: number[],
    public oracleProgramId: PublicKey
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

    const transaction = new Transaction().add(
      await initLendingMarketIx(lendingMarketOwner.publicKey, QUOTE_CURRENCY, lendingMarketPubkey, oracleProgramId)
    );
    transaction.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
    transaction.feePayer = lendingMarketOwner.publicKey;

    await web3.sendAndConfirmTransaction(connection, transaction, [lendingMarketOwner, lendingMarketKeypair]);

    return new TestLendingMarket(
      lendingMarketPubkey,
      lendingMarketOwner,
      lendingMarketAuthority,
      QUOTE_CURRENCY,
      oracleProgramId
    );
  }

  async getState() {
    return await program.account.lendingMarket.fetch(this.pubkey);
  }

  async validateState() {
    const lendingMarket = await this.getState();
    expect(lendingMarket.version).toBe(Number(IDL.constants[0].value));
    expect(lendingMarket.owner).toEqual(this.owner.publicKey);
    expect(lendingMarket.quoteCurrency).toEqual(this.quoteCurrency);
  }
}
