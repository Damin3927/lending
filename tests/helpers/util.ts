import { Keypair } from "@solana/web3.js";
import fs from "fs";

export const readKeypairFile = (path: string) =>
  Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(path).toString())));

export const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
