import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import type { Staking } from "../target/types/staking.ts";

describe("staking", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.staking as Program<Staking>;

  // it("Is initialized!", async () => {
  //   // Add your test here.
  //   const tx = await program.methods.initializePool().rpc();
  //   console.log("Your transaction signature", tx);
  // });
});
