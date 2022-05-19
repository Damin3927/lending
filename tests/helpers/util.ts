import { IDL, LendingAnchor } from "../../target/types/lending_anchor";
import { Keypair, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";
import fs from "fs";
import { connection } from "../common";

export const readKeypairFile = (path: string) =>
  Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(path).toString())));

export const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export const airdropTo = async (to: PublicKey, sol: number) =>
  await connection.confirmTransaction(await connection.requestAirdrop(to, LAMPORTS_PER_SOL * sol));

export const generateWealthKeypair = async () => {
  const keypair = Keypair.generate();
  await airdropTo(keypair.publicKey, 100);
  return keypair;
};

export const errorOf = (errCode: number) => `custom program error: 0x${errCode.toString(16)}`;
export const customErrorOf = (name: (LendingAnchor["errors"] extends (infer U)[] ? U : never)["name"]) =>
  errorOf(IDL.errors.find((value) => value.name === name).code);

export const signatureVerificationError = "Signature verification failed";
