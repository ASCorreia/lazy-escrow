import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LazyEscrow } from "../target/types/lazy_escrow";
import {
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  createInitializeMint2Instruction,
  createMint,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
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

  let mintA: anchor.web3.PublicKey;
  let mintB: anchor.web3.PublicKey;

  let userAtaA: anchor.web3.PublicKey;
  let userAtaB: anchor.web3.PublicKey;

  let vault: anchor.web3.PublicKey;

  const escrow = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("escrow"), provider.publicKey.toBuffer()], program.programId)[0];

  it("Mint Tokens to Maker and Taker!", async () => {
    mintA = await createMint(provider.connection, payer.payer, provider.publicKey, provider.publicKey, 6);
    console.log("\nmintA", mintA.toBase58());

    vault = getAssociatedTokenAddressSync(mintA, escrow, true);

    mintB = await createMint(provider.connection, payer.payer, provider.publicKey, provider.publicKey, 6);
    console.log("mintB", mintB.toBase58());

    userAtaA = (await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, mintA, provider.publicKey)).address;
    userAtaB = (await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, mintB, provider.publicKey)).address; // Placeholder to be replaced with taker's ATA

    await mintTo(provider.connection, payer.payer, mintA, userAtaA, payer.payer, 100000000);
    console.log("\nTokens minted to userAtaA ", userAtaA.toBase58());

    await mintTo(provider.connection, payer.payer, mintB, userAtaB, payer.payer, 100000000); // Placeholder to be replaced with taker's ATA
    console.log("Tokens minted to userAtaB ", userAtaB.toBase58());
  });

  it("Make!", async () => {
    // Add your test here.
    const tx = await program.methods.make(new anchor.BN(10), new anchor.BN(20))
    .accountsPartial({
      escrow: escrow,
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

  it("Refund!", async () => {
    // Add your test here.
    const tx = await program.methods.refund()
    .accountsPartial({
      escrow: escrow,
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
});
