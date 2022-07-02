import * as _anchor from "@project-serum/anchor";
import { LendingAnchor, IDL } from "../target/types/lending_anchor";
import { sleep } from "./helpers/util";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import { TextEncoder } from "node:util";

export const anchor = _anchor;
export const program = anchor.workspace.LendingAnchor as _anchor.Program<LendingAnchor>;
export { IDL, LendingAnchor };
export const provider = program.provider;
export const connection = provider.connection;

_anchor.setProvider(provider);

export const QUOTE_CURRENCY: number[] = Array.from(
  new TextEncoder().encode("USD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0")
);

it("exports a correct quote currency", function () {
  expect(QUOTE_CURRENCY.length).toBe(32);
});

const payer = anchor.web3.Keypair.generate();

let initialized = false;
(async () => {
  await connection.confirmTransaction(await connection.requestAirdrop(payer.publicKey, LAMPORTS_PER_SOL * 2));
  initialized = true;
})();

export const getPayer = async () => {
  while (!initialized) {
    await sleep(100);
  }
  return payer;
};
