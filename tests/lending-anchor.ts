import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { LendingAnchor } from "../target/types/lending_anchor";

describe("lending-anchor", function () {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.LendingAnchor as Program<LendingAnchor>;

  it("Is initialized!", async function () {
    // Add your test here.
    expect(1).toBe(1);
  });
});
