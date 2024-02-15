import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { TallyClob } from "../../target/types/tally_clob";

export function getProgram(): Program<TallyClob> {
    const provider = AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.TallyClob as Program<TallyClob>;
    return program
}