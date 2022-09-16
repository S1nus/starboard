import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Starboard } from "../target/types/starboard";
import {
  Keypair,
  PublicKey,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  createFeed,
  createRound,
  startFeed,
  startStaking,
  stake,
} from './utils';
import { 
  TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
} from '@solana/spl-token';
import {NATIVE_MINT, TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount, createAssociatedTokenAccountInstruction, getAssociatedTokenAddress, createSyncNativeInstruction, getAccount} from "@solana/spl-token";

describe("starboard", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Starboard as Program<Starboard>;
  const payer = (program.provider as anchor.AnchorProvider).wallet.payer;

  it("Initializes a feed", async () => {

    const voterAta = await getOrCreateAssociatedTokenAccount(program.provider.connection, payer, new PublicKey("So11111111111111111111111111111111111111112"), payer.publicKey);
    const solTransferTransaction = new Transaction()
      .add(
        SystemProgram.transfer({
          fromPubkey: payer.publicKey,
          toPubkey: voterAta.address,
          lamports: LAMPORTS_PER_SOL
        }),
        createSyncNativeInstruction(
          voterAta.address
        )
      );
    await sendAndConfirmTransaction(program.provider.connection, solTransferTransaction, [payer]);

    const feed = await createFeed(program, payer, "SOL/USD", 30);
    console.log(`Feed: ${feed}`);

    let rounds = [];
    for (var i = 0; i<5; i++) {
      let key = await createRound(program, payer, feed, i);
      rounds[i] = key;
      console.log(`round ${i}: ${key}`);
    }

    await startStaking(program, feed, rounds[0]);
    await stake(program, payer, feed);

  });
});
