import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { LendingAnchor } from "../target/types/lending_anchor";

describe("lending-anchor", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.LendingAnchor as Program<LendingAnchor>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
