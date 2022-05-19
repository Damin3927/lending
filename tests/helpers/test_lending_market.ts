import { connection, program, QUOTE_CURRENCY } from "../common";
import { airdropTo, readKeypairFile } from "./util";
import { PublicKey, Keypair, Transaction } from "@solana/web3.js";
import { web3 } from "@project-serum/anchor";
import { initLendingMarketIx } from "../instructions/init_lending_market";
import { IDL } from "../../target/types/lending_anchor";
import { setLendingMarketIx } from "../instructions/set_lending_market_owner";

export class TestLendingMarket {
  /**
   * @param keypair - The keypair of this account
   * @param _owner - The owner of this account
   * @param authority - The owner authority which can add new reserves
   * @param oracleProgramId - The oracle program id (Pyth)
   */
  constructor(
    public readonly keypair: Keypair,
    private _owner: Keypair,
    public readonly authority: PublicKey,
    public readonly quoteCurrency: number[],
    public readonly oracleProgramId: PublicKey
  ) {}

  static async init() {
    const owner = readKeypairFile("tests/fixtures/lending_market_owner.json");
    await airdropTo(owner.publicKey, 1);
    const oracleProgramId = readKeypairFile("tests/fixtures/oracle_program_id.json").publicKey;

    const keypair = Keypair.generate();
    const pubkey = keypair.publicKey;
    const [authority, _bumpSeed] = PublicKey.findProgramAddressSync([pubkey.toBuffer()], program.programId);

    return new TestLendingMarket(keypair, owner, authority, QUOTE_CURRENCY, oracleProgramId);
  }

  async createLendingMarket() {
    const transaction = new Transaction().add(
      await initLendingMarketIx(this._owner.publicKey, this.quoteCurrency, this.keypair.publicKey, this.oracleProgramId)
    );
    transaction.feePayer = this._owner.publicKey;

    await web3.sendAndConfirmTransaction(connection, transaction, [this._owner, this.keypair]);
    return this;
  }

  async setNewOwner(
    newOwner: Keypair,
    options?: {
      currentOwner?: Keypair;
      signer?: Keypair;
    }
  ) {
    const currentOwner = options?.currentOwner ?? this._owner;
    const signer = options?.signer ?? currentOwner;

    const transaction = new Transaction().add(
      await setLendingMarketIx(currentOwner, this.keypair.publicKey, newOwner.publicKey)
    );
    transaction.feePayer = signer.publicKey;
    await web3.sendAndConfirmTransaction(connection, transaction, [signer]);

    this._owner = newOwner;
    return this;
  }

  async getState() {
    return await program.account.lendingMarket.fetch(this.keypair.publicKey);
  }

  async validateState() {
    const lendingMarket = await this.getState();
    expect(lendingMarket.version).toBe(Number(IDL.constants[0].value));
    expect(lendingMarket.owner).toEqual(this._owner.publicKey);
    expect(lendingMarket.quoteCurrency).toEqual(this.quoteCurrency);
  }
}
