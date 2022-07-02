import { PublicKey, SystemProgram, AccountInfo } from "@solana/web3.js";
import { BN } from "@project-serum/anchor";
import { readKeypairFile } from "./util";
import { readFileSync } from "fs";
import { parsePriceData } from "@pythnetwork/client";

export const SOL_PYTH_PRODUCT = "3Mnn2fX6rQyUsyELYms1sBJyChWofzSNRoqYzvgMVz5E";
export const SOL_PYTH_PRICE = "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix";

export class TestOracle {
  constructor(
    public readonly productPubkey: PublicKey,
    public readonly pricePubkey: PublicKey,
    public readonly price: BN
  ) {}

  public static addSolOracle() {
    return this.addOracle(new PublicKey(SOL_PYTH_PRODUCT), new PublicKey(SOL_PYTH_PRICE), new BN(20));
  }

  public static addOracle(productPubkey: PublicKey, pricePubkey: PublicKey, price: BN) {
    const oracleProgramId = readKeypairFile("tests/fixtures/oracle_program_id.json");

    // add pyth product account
    // TODO

    // add pyth price account
    const filename = `tests/fixtures/${pricePubkey.toString()}.bin`;
    const pythPriceData = readFileSync(filename);

    const pythPrice = parsePriceData(pythPriceData);

    return new TestOracle(productPubkey, pricePubkey, price);
  }
}
