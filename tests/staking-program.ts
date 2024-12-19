import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StakingProgram } from "../target/types/staking_program";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";

describe("staking-program", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = provider.wallet as anchor.Wallet;
  const connection = new Connection("https://api.devnet.solana.com/", "confirmed"); 

  const mintKeypair = Keypair.generate();
  console.log(mintKeypair);

  const program = anchor.workspace.StakingProgram as Program<StakingProgram>;

  async function createMintToken() {
    const mint = await createMint(
      connection,
      payer.payer,
      payer.publicKey, // mint authority
      payer.publicKey, // freeze authority
      9, // 9 decimals
      mintKeypair
    )
    console.log('mint',mint);
  }

  it("Is initialized!", async () => {
    await createMintToken();

    let [vaultAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault")],
      program.programId
    )
    console.log(vaultAccount,'vaultAccount');
    

    const tx = await program.methods.initialize()
    .accounts({
      signer: payer.publicKey,
      tokenVaultAccount: vaultAccount,
      mint: mintKeypair.publicKey
    })
    .rpc();

    console.log("Your init transaction signature", tx);
  });

  it("get stakeInfo", async () => {
    let [stakeInfo] =  PublicKey.findProgramAddressSync(
      [Buffer.from("stake_info"), payer.publicKey.toBuffer()],
      program.programId
    )
    const account = await program.account.stakeInfo.fetch(stakeInfo)
    console.log(account,'stakeInfo');
  })

  it("stake", async () => {
    let userTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintKeypair.publicKey,
      payer.publicKey
    );

    await mintTo(
      connection,
      payer.payer,
      mintKeypair.publicKey,
      userTokenAccount.address,
      payer.payer,
      1e11
    )

    let [vaultAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault")],
      program.programId
    )
    console.log(vaultAccount,'vaultAccount');

    let [stakeInfo] =  PublicKey.findProgramAddressSync(
      [Buffer.from("stake_info"), payer.publicKey.toBuffer()],
      program.programId
    )

    let [stakeAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("token"), payer.publicKey.toBuffer()],
      program.programId
    )

    await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintKeypair.publicKey,
      payer.publicKey
    )

    const tx = await program.methods
    .stake(new anchor.BN(10))
    .signers([payer.payer])
    .accounts({
      stakeInfoAccount: stakeInfo,
      stakeAccount: stakeAccount,
      userTokenAccount: userTokenAccount.address,
      mint: mintKeypair.publicKey,
      signer: payer.publicKey,
    })
    .rpc()

    console.log("Your stake transaction signature", tx);
  });

  it("destake", async () => {
    let userTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintKeypair.publicKey,
      new PublicKey('E1g1o3HkZ8n9Ce7ic4peH7q6Ag8UYdwDfEzDR6VjyUqN')
    );

    let [stakeInfo] =  PublicKey.findProgramAddressSync(
      [Buffer.from("stake_info"), payer.publicKey.toBuffer()],
      program.programId
    )

    let [stakeAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("token"), payer.publicKey.toBuffer()],
      program.programId
    )

    let [vaultAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault")],
      program.programId
    )

    console.log(vaultAccount,'vaultAccount');

    await mintTo(
      connection,
      payer.payer,
      mintKeypair.publicKey,
      vaultAccount,
      payer.payer,
      1e11
    );

    const tx = await program.methods
    .destake()
    .signers([payer.payer])
    .accounts({
      stakeAccount:stakeAccount,
      stakeInfoAccount:stakeInfo,
      userTokenAccount:userTokenAccount.address,
      tokenVaultAccount: vaultAccount,
      signer: payer.publicKey,
      mint: mintKeypair.publicKey,
    })
    .rpc()

    console.log("Your destake transaction signature", tx);
  })
});
