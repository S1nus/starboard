import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
const webcrypto = require('crypto').webcrypto;

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
    .startStaking(roundNum)
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
  const feedData = await program.account.feed.fetch(feed);
  const round = feedData.stakingRound;
  const roundData = await program.account.round.fetch(round);
  console.log(roundData);
  console.log(feedData);
  const roundHeight = roundData.roundHeight;
  console.log(roundBuf);
  const roundNum = roundData.num;
  const [escrowKey] = await PublicKey.findProgramAddress(
    [
      Buffer.from("Escrow"),
      payer.publicKey.toBytes(),
      feed.toBytes(),
      roundHeight
    ],
    program.programId
  );
  console.log(escrowKey.toBase58());
  console.log([
      Buffer.from("Escrow"),
      payer.publicKey.toBytes(),
      feed.toBytes(),
      roundBuf
    ]);
  console.log({
    escrow: escrowKey.toBase58(),
    voter: payer.publicKey.toBase58(),
    feed: feed.toBase58(),
    round: round.toBase58(),
    systemProgram: SystemProgram.programId.toBase58(),
    rent: SYSVAR_RENT_PUBKEY.toBase58(),
  });
  const tx = await program
    .methods
    .stake(roundNum, roundHeight)
    .accounts({
      escrow: escrowKey,
      voter: payer.publicKey,
      feed: feed,
      round: round,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
    })
    .signers([payer])
    .rpc({skipPreflight: true});
}
