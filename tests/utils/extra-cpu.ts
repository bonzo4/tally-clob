import * as anchor from "@coral-xyz/anchor";

export const additionalComputeBudgetInstruction =
anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
  units: 1_400_000,
});