import { Keypair, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";
import fs from "fs";
import { connection } from "../common";

export const readKeypairFile = (path: string) =>
  Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(path).toString())));

export const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export const airdropTo = async (to: PublicKey, sol: number) =>
  await connection.confirmTransaction(await connection.requestAirdrop(to, LAMPORTS_PER_SOL * sol));
