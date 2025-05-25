import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LazyEscrow } from "../target/types/lazy_escrow";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("lazy-escrow", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.lazyEscrow as Program<LazyEscrow>;

  const payer = provider.wallet as NodeWallet;

  const taker = anchor.web3.Keypair.generate();

  let mintA: anchor.web3.PublicKey;
  let mintB: anchor.web3.PublicKey;

  let userAtaA: anchor.web3.PublicKey;
  let userAtaB: anchor.web3.PublicKey;

  let takerAtaA: anchor.web3.PublicKey;
  let takerAtaB: anchor.web3.PublicKey;

  let vault: anchor.web3.PublicKey;

  const escrow = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("escrow"), provider.publicKey.toBuffer()], program.programId)[0];

  // it("Request airdrop to taker!", async () => {
  //   const signature = await provider.connection.requestAirdrop(taker.publicKey, 10000000000);
  //   await provider.connection.confirmTransaction(signature);
  //   console.log("\nAirdrop to taker successful - Signature", signature);
  // });

  it("Mint Tokens to Maker and Taker!", async () => {
    mintA = await createMint(provider.connection, payer.payer, provider.publicKey, provider.publicKey, 6);
    console.log("\nmintA", mintA.toBase58());

    vault = getAssociatedTokenAddressSync(mintA, escrow, true);

    mintB = await createMint(provider.connection, payer.payer, provider.publicKey, provider.publicKey, 6);
    console.log("mintB", mintB.toBase58());

    userAtaA = (await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, mintA, provider.publicKey)).address;
    userAtaB = (await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, mintB, provider.publicKey)).address; 
    
    takerAtaA = (await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, mintA, taker.publicKey)).address;
    takerAtaB = (await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, mintB, taker.publicKey)).address;

    await mintTo(provider.connection, payer.payer, mintA, userAtaA, payer.payer, 100000000);
    console.log("\nTokens minted to userAtaA ", userAtaA.toBase58());

    await mintTo(provider.connection, payer.payer, mintB, userAtaB, payer.payer, 100000000);
    console.log("Tokens minted to userAtaB ", userAtaB.toBase58());

    await mintTo(provider.connection, payer.payer, mintA, takerAtaA, payer.payer, 100000000);
    console.log("\nTokens minted to takerAtaA ", takerAtaA.toBase58());
    await mintTo(provider.connection, payer.payer, mintB, takerAtaB, payer.payer, 100000000);
    console.log("Tokens minted to takerAtaB ", takerAtaB.toBase58());
  });

  it("Make!", async () => {
    // Add your test here.
    const tx = await program.methods.make(new anchor.BN(10), new anchor.BN(20))
    .accountsPartial({
      escrow,
      maker: provider.publicKey,
      tokenA: mintA,
      tokenB: mintB,
      ataMakerTokenA: userAtaA,
      vaultTokenA: vault,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

    console.log("\nMake method successfully invoked - Transaction signature", tx);
  });

  xit("Refund!", async () => {
    // Add your test here.
    const tx = await program.methods.refund()
    .accountsPartial({
      escrow,
      maker: provider.publicKey,
      tokenA: mintA,
      ataMakerTokenA: userAtaA,
      vaultTokenA: vault,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

    console.log("\nRefund method successfully invoked - Transaction signature", tx);
  });

  it("Take!", async () => {
    // Add your test here.
    const tx = await program.methods.take()
    .accountsPartial({
      taker: taker.publicKey,
      maker: provider.publicKey,
      tokenA: mintA,
      tokenB: mintB,
      vaultTokenA: vault,
      ataMakerTokenB: userAtaB,
      ataTakerTokenA: takerAtaA,
      ataTakerTokenB: takerAtaB,
      escrow,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([taker])
    .rpc();

    console.log("\nTake method successfully invoked - Transaction signature", tx);
  });
});
