import { IDL, LendingAnchor } from "../../target/types/lending_anchor";
import { Keypair, PublicKey, LAMPORTS_PER_SOL, sendAndConfirmTransaction, SystemProgram } from "@solana/web3.js";
import fs from "fs";
import { connection, program } from "../common";
import { BN } from "@project-serum/anchor";
import { createAccount, mintTo } from "@solana/spl-token";

export const readKeypairFile = (path: string) =>
  Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(path).toString())));

export const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export const airdropTo = async (to: PublicKey, sol: number) =>
  await connection.confirmTransaction(await connection.requestAirdrop(to, LAMPORTS_PER_SOL * sol));

export const generateWealthyKeypair = async () => {
  const keypair = Keypair.generate();
  await airdropTo(keypair.publicKey, 100);
  return keypair;
};

export const createAndMintToTokenAccount = async (
  amount: number,
  owner: PublicKey,
  payer: Keypair,
  mintPubkey: PublicKey,
  mintAuthority: PublicKey = null
) => {
  const account = await createAccount(connection, payer, mintPubkey, owner);
  if (mintAuthority === null) {
    // wsol
    await airdropTo(account, amount);
  } else {
    await mintTo(connection, payer, mintPubkey, account, mintAuthority, amount);
  }
  return account;
};

export const errorOf = (errCode: number) => `custom program error: 0x${errCode.toString(16)}`;
export const customErrorOf = (name: (LendingAnchor["errors"] extends (infer U)[] ? U : never)["name"]) =>
  errorOf(IDL.errors.find((value) => value.name === name).code);

export const signatureVerificationError = "Signature verification failed";

export const constantOf: (name: LendingAnchor["constants"][number]["name"]) => number | BN = (name) => {
  const constant = IDL.constants.find((constant) => constant.name === name);
  if (["u64"].includes(constant.type)) {
    return new BN(constant.value);
  }
  if (["u8"].includes(constant.type)) {
    return Number(constant.value);
  }
  throw new Error("unknown type");
};
