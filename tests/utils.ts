import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
const webcrypto = require('crypto').webcrypto;
import { 
  TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
} from '@solana/spl-token';
import {
  Keypair,
  PublicKey,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";

export async function createFeed(program: Program, payer: Keypair, desc: string, updateInterval: number): PublicKey {
  const id = Buffer.alloc(32);
  webcrypto.getRandomValues(id);
  const [feedKey] = await PublicKey.findProgramAddress(
    [
      Buffer.from("Feed"),
      id
    ],
    program.programId
  );
  const tx = await program
      .methods
      .initFeed(id, Buffer.from(desc.padEnd(32,"\0")), updateInterval)
      .accounts({
        feed: feedKey,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([payer])
      .rpc({skipPreflight: true});
  return feedKey;
}

export async function createRound(program: Program, payer: Keypair, feed: PublicKey, num: number): PublicKey {
  const [roundKey] = await PublicKey.findProgramAddress(
    [
      Buffer.from("Round"),
      feed.toBytes(),
      Buffer.from([num]),
    ],
    program.programId
  );
  const tx = await program
      .methods
      .initRound(num)
      .accounts({
        round: roundKey,
        feed: feed,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([payer])
      .rpc({skipPreflight: true});
  return roundKey;
}

export async function startFeed(program: Program, feed: PublicKey) {
  const tx = await program
      .methods
      .startFeed()
      .accounts({
        feed: feed,
      })
      .rpc({skipPreflight: true});
}

export async function startStaking(program: Program, feed: PublicKey, round: PublicKey) {
  const roundData = await program.account.round.fetch(round);
  const feedData = await program.account.feed.fetch(feed);
  const roundNum = roundData.num;
  const oldRound = feedData.stakingRound;
  const tx = await program
    .methods
    .startStaking()
    .accounts({
      feed: feed,
      round: round,
    })
    .remainingAccounts([{
      isSigner: false,
      isMutable: false,
      pubkey: oldRound,
    }])
    .rpc({skipPreflight: true});
}

export async function stake(program: Program, payer: Keypair, feed: PublicKey) {

  const voterAta = await getOrCreateAssociatedTokenAccount(program.provider.connection, payer, new PublicKey("So11111111111111111111111111111111111111112"), payer.publicKey);
  const feedData = await program.account.feed.fetch(feed);
  const round = feedData.stakingRound;
  const roundData = await program.account.round.fetch(round);
  const [escrowKey] = await PublicKey.findProgramAddress(
    [
      Buffer.from("Escrow"),
      payer.publicKey.toBytes(),
      round.toBytes(),
    ],
    program.programId
  );

  const [escrowToken] = await PublicKey.findProgramAddress(
    [
      Buffer.from("EscrowToken"),
      escrowKey.toBytes()
    ],
    program.programId
  );

  const [programAsSigner] = await PublicKey.findProgramAddress(
    [
      Buffer.from("program"),
      Buffer.from("signer"),
    ],
    program.programId
  );

  const tx = await program
    .methods
    .stake()
    .accounts({
      escrow: escrowKey,
      escrowToken: escrowToken,
      voter: payer.publicKey,
      voterTokenAccount: voterAta.address,
      feed: feed,
      round: round,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
      nativeMint: new PublicKey("So11111111111111111111111111111111111111112"),
      programAsSigner: programAsSigner,
    })
    .signers([payer])
    .rpc({skipPreflight: true});
}
