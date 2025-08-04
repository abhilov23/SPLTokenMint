import * as anchor from "@coral-xyz/anchor";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { Keypair, SystemProgram, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";
import BN from "bn.js"

describe("custom-token", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.CustomToken;

  it("creates a mint with no extensions", async () => {
    // Step 1: Create keypairs for authorities and recipient
    const mintAuthority = Keypair.generate();
    const recipient = Keypair.generate();

    // Step 2: Derive mint PDA (this might be what your program expects)
    // Common seed patterns - try different ones based on your Rust program
    const [mintPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint"), provider.publicKey.toBuffer()], // or other seeds
      program.programId
    );

    console.log("Using mint PDA:", mintPda.toBase58());

    // Alternative: If PDA doesn't work, try the generated keypair approach
    const mint = Keypair.generate();
    console.log("Generated mint:", mint.publicKey.toBase58());

    // Step 3: Airdrop SOL to mintAuthority & recipient
    for (let key of [mintAuthority, recipient]) {
      const sig = await provider.connection.requestAirdrop(
        key.publicKey,
        LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(sig);
    }

    // Step 4: Try with PDA first
    try {
      console.log("Trying with PDA mint...");
      
      const recipientAta = getAssociatedTokenAddressSync(
        mintPda,
        recipient.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      await program.methods
        .createTokenWithExtensions({
          decimals: 6,
          transferFeeBasisPoints: 0,
          maxFee: new BN(0),
          requireMemo: false,
          defaultFrozen: false,
          immutableOwner: false,
          initialSupply: new BN(0),
        })
        .accounts({
          payer: provider.publicKey,
          mint: mintPda, // Use PDA
          mintAuthority: mintAuthority.publicKey,
          recipient: recipient.publicKey,
          recipientAta,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc();

      console.log("✅ PDA approach worked!");
      
      // Validate
      const mintAccount = await provider.connection.getAccountInfo(mintPda);
      assert.ok(mintAccount !== null);
      
    } catch (error) {
      console.log("PDA approach failed:", error.message);
      console.log("Trying with generated keypair + explicit creation...");
      
      // Step 5: If PDA doesn't work, your program might need the mint to be pre-created
      // Create the mint account first
      const createMintIx = SystemProgram.createAccount({
        fromPubkey: provider.publicKey,
        newAccountPubkey: mint.publicKey,
        lamports: await provider.connection.getMinimumBalanceForRentExemption(165), // Token-2022 mint size
        space: 165,
        programId: TOKEN_2022_PROGRAM_ID,
      });

      const recipientAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        recipient.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      // Create transaction with mint creation + your program call
      const tx = new anchor.web3.Transaction()
        .add(createMintIx)
        .add(
          await program.methods
            .createTokenWithExtensions({
              decimals: 6,
              transferFeeBasisPoints: 0,
              maxFee: new BN(0),
              requireMemo: false,
              defaultFrozen: false,
              immutableOwner: false,
              initialSupply: new BN(0),
            })
            .accounts({
              payer: provider.publicKey,
              mint: mint.publicKey,
              mintAuthority: mintAuthority.publicKey,
              recipient: recipient.publicKey,
              recipientAta,
              tokenProgram: TOKEN_2022_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
              rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            })
            .instruction()
        );

      // Send transaction with both mint creation and program call
      await provider.sendAndConfirm(tx, [mint]);

      // Validate
      const mintAccount = await provider.connection.getAccountInfo(mint.publicKey);
      assert.ok(mintAccount !== null);
    }
  });
});