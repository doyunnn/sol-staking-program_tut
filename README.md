# Solana Staking

Date: 2024년 8월 12일 → 2024년 8월 14일
Type: Block Chain
상태: Done

<aside>
💡 스테이킹 프로그램을 구현하고 DApp 구현을 목표로 한다.

</aside>

## 1. 기능 상세

---

1. 솔라나 스테이킹 프로그램 (Anchor)
2. 지갑 연동 (dapp)
3. 스테이킹, 언 스테이킹 (dapp)
    1. 상태 확인
    2. 수량 확인
    3. 예상 보상 수량 확인
4. 토큰 민팅 (10개) (dapp)

## 2. 스테이킹 프로그램 구현

---

#### 스테이킹 프로그램 기본 구조
<img width="898" alt="%E1%84%89%E1%85%B3%E1%84%8F%E1%85%B3%E1%84%85%E1%85%B5%E1%86%AB%E1%84%89%E1%85%A3%E1%86%BA_2024-08-23_10 17 35" src="https://github.com/user-attachments/assets/b0535185-6d6e-45e3-aa39-bdd48d71f029" />

스테이킹 프로그램 기본 구조

<aside>
👉🏻 ***4가지의 PDA를 사용해 스테이킹 프로그램을 구현함.***

1. User Associated Token Account (유저의 토큰 계정)
2. Stake Info Account (스테이킹 정보 계정)
3. User Stake Account (유저 스테이킹 계정)
4. Vault Account (토큰 관리 계정)

</aside>

### DApp 로직

1. 유저가 지갑을 연동 하면 User ATA와 Stake Info PDA에서 정보를 가져온다.
    1. User ATA에서는 토큰 보유 수량을
    2. Stake Info PDA에서는 스테이킹 상태와 시간을 가져온다.
2. ATA에서 토큰을 보유하지 않았을 경우 토큰을 민팅할 수 있도록 ‘토큰 얻기 (10개 제공)’ 버튼을 노출한다.
3. 
    ![스크린샷 2024-08-23 10.38.11.png](https://github.com/user-attachments/assets/5775c4cd-14aa-479e-bfd3-9ec65128fc29)
    
4. 민팅을 통해 유저가 토큰을 보유했다면 Stake Info PDA로 부터 가져온 스테이킹 상태를 보여준다.
5. 스테이킹을 하지 않았다면 스테이킹을 유도하는 ‘Stake’ 버튼을 노출한다.

   ![스크린샷 2024-08-23 10.39.03.png](https://github.com/user-attachments/assets/db3c5bba-5b3b-42e3-8069-8232a503e758)
    
7. 원하는 스테이킹 수량을 입력하고 스테이킹 버튼을 클릭 시 연동된 플랫폼의 트랜잭션 사인 프로세스가 진행되며 확인 후 스테이킹이 진행된다.

   ![스크린샷 2024-08-23 10.40.49.png](https://github.com/user-attachments/assets/7da20063-5f73-4837-9779-a0f2cfdf7d23)
    
9. 스테이킹을 진행한 유저는 Stake Info PDA와 User Stake PAD로 부터 정보를 가져와 스테이킹 진행 상테를 확인할 수 있다.
    1. Stake Info에서는 상태와 시간을
    2. User Stake에서는 스테이킹 수량을
  ![스크린샷 2024-08-23 10.41.38.png](https://github.com/user-attachments/assets/c409dae3-c25a-4a8a-b16d-719d0d9a72b2)
        
10. 언 스테이킹을 원하는 유저는 ‘Unstake’ 버튼을 통해 트랜잭션을 만들 수 있다.
11. 확인 후 이전(4번)과 같은 상태로 변경된다.

### Anchor 로직

1. Seeds
    - constants
        
        ```rust
        pub mod constants {
            pub const VAULT_SEED: &[u8] = b"vault";
            pub const STAKE_INFO_SEED: &[u8] = b"stake_info";
            pub const TOKEN_SEED: &[u8] = b"token";
        }
        ```
        
2. Program
    - Stake
        
        ```rust
         pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
                let stake_info = &mut ctx.accounts.stake_info_account;
        
                if stake_info.is_staked {
                    return Err(ErrorCode::IsStaked.into());
                }
        
                if amount == 0 {
                    return Err(ErrorCode::NoTokens.into());
                }
        
                let clock = Clock::get()?;
        
                stake_info.stake_at_slot = clock.slot;
                stake_info.is_staked = true;
        
                let stake_amount = (amount)
                    .checked_mul(10u64.pow(ctx.accounts.mint.decimals as u32))
                    .unwrap();
        
                transfer(
                    CpiContext::new(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.user_token_account.to_account_info(),
                            to: ctx.accounts.stake_account.to_account_info(),
                            authority: ctx.accounts.signer.to_account_info(),
                        },
                    ),
                    stake_amount,
                )?;
        
                Ok(())
            }
        ```
        
    - UnStake
        
        ```rust
        pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
                let stake_info = &mut ctx.accounts.stake_info_account;
        
                if !stake_info.is_staked {
                    return Err(ErrorCode::NotStaked.into());
                }
        
                let clock = Clock::get()?;
        
                let stake_amount = ctx.accounts.stake_account.amount;
        
                // 스테이킹 기간 계산 (슬롯 단위)
                let slots_staked = clock.slot.saturating_sub(stake_info.stake_at_slot);
        
                // 1분을 슬롯으로 변환 (1분 * 60초 * 1000ms / 400ms)
                // let thirty_minutes_in_slots = 30 * 60 * 1000 / 400;
                let thirty_minutes_in_slots = 1 * 60 * 1000 / 400;
        
                // 분당 1%의 보상률 계산
                let periods = slots_staked as f64 / thirty_minutes_in_slots as f64;
                let reward_rate = 0.001; // 0.1%
        
                // 보상 계산
                let reward = (stake_amount as f64 * reward_rate * periods) as u64;
        
                let bump = *ctx.bumps.get("token_vault_account").unwrap();
                let signer: &[&[&[u8]]] = &[&[constants::VAULT_SEED, &[bump]]];
        
                transfer(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.token_vault_account.to_account_info(),
                            to: ctx.accounts.user_token_account.to_account_info(),
                            authority: ctx.accounts.token_vault_account.to_account_info(),
                        },
                        signer,
                    ),
                    reward,
                )?;
        
                let staker = ctx.accounts.signer.key();
                let bump = *ctx.bumps.get("stake_account").unwrap();
                let signer: &[&[&[u8]]] = &[&[constants::TOKEN_SEED, staker.as_ref(), &[bump]]];
        
                transfer(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.stake_account.to_account_info(),
                            to: ctx.accounts.user_token_account.to_account_info(),
                            authority: ctx.accounts.stake_account.to_account_info(),
                        },
                        signer,
                    ),
                    stake_amount,
                )?;
        
                stake_info.is_staked = false;
                stake_info.stake_at_slot = clock.slot;
        
                Ok(())
            }
            
        ```
        
3. Accounts
    - Stake
        
        ```rust
        #[derive(Accounts)]
        pub struct Stake<'info> {
            #[account(mut)]
            pub signer: Signer<'info>,
        
            #[account(
                init_if_needed,
                seeds = [constants::STAKE_INFO_SEED, signer.key().as_ref()],
                bump,
                payer = signer,
                space = 8 + std::mem::size_of::<StakeInfo>(),
            )]
            pub stake_info_account: Account<'info, StakeInfo>,
        
            #[account(
                init_if_needed,
                seeds = [constants::TOKEN_SEED, signer.key().as_ref()],
                bump,
                payer = signer,
                token::mint = mint,
                token::authority = stake_account,
            )]
            pub stake_account: Account<'info, TokenAccount>,
        
            #[account(
                mut,
                associated_token::mint = mint,
                associated_token::authority = signer,
            )]
            pub user_token_account: Account<'info, TokenAccount>,
        
            pub mint: Account<'info, Mint>,
            pub token_program: Program<'info, Token>,
            pub associated_token_program: Program<'info, AssociatedToken>,
            pub system_program: Program<'info, System>,
        }
        ```
        
    - Unstake
        
        ```rust
        
        #[derive(Accounts)]
        pub struct Destake<'info> {
            #[account(mut)]
            pub signer: Signer<'info>,
        
            #[account(
                mut,
                seeds = [constants::VAULT_SEED],
                bump,
            )]
            pub token_vault_account: Account<'info, TokenAccount>,
        
            #[account(
                mut,
                seeds = [constants::STAKE_INFO_SEED, signer.key().as_ref()],
                bump,
            )]
            pub stake_info_account: Account<'info, StakeInfo>,
        
            #[account(
                mut,
                seeds = [constants::TOKEN_SEED, signer.key().as_ref()],
                bump,
            )]
            pub stake_account: Account<'info, TokenAccount>,
        
            #[account(
                mut,
                associated_token::mint = mint,
                associated_token::authority = signer,
            )]
            pub user_token_account: Account<'info, TokenAccount>,
        
            pub mint: Account<'info, Mint>,
            pub token_program: Program<'info, Token>,
            pub associated_token_program: Program<'info, AssociatedToken>,
            pub system_program: Program<'info, System>,
        }
        ```
        
    - Stake Info
        
        ```rust
        
        #[account]
        pub struct StakeInfo {
            pub stake_at_slot: u64,
            pub is_staked: bool,
        }
        ```
        
4. Test
    - staking program
        
        ```rust
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
        
        ```
        

### Stake

1. stake info 계정에 스테이킹 시간과 상태를 변경한다.
    
    ```rust
    let clock = Clock::get()?;
    
    stake_info.stake_at_slot = clock.slot;
    stake_info.is_staked = true;
    ```
    
2. Transfer CPI를 통해 ATA에서 stake account로 토큰을 전송한다.
    
    ```rust
    transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.user_token_account.to_account_info(),
                        to: ctx.accounts.stake_account.to_account_info(),
                        authority: ctx.accounts.signer.to_account_info(),
                    },
                ),
                stake_amount,
            )?;
    ```
    

### UnStake

1. stake info 계정에서 스테이킹 시간과 상태를 가져와 보상을 계산한다.
    
    ```
    let clock = Clock::get()?;
    
    let stake_amount = ctx.accounts.stake_account.amount;
    
    // 스테이킹 기간 계산 (슬롯 단위)
    let slots_staked = clock.slot.saturating_sub(stake_info.stake_at_slot);
    
    // 1분을 슬롯으로 변환 (1분 * 60초 * 1000ms / 400ms)
    // let thirty_minutes_in_slots = 30 * 60 * 1000 / 400;
    let thirty_minutes_in_slots = 1 * 60 * 1000 / 400;
    
    // 분당 1%의 보상률 계산
    let periods = slots_staked as f64 / thirty_minutes_in_slots as f64;
    let reward_rate = 0.001; // 0.1%
    
    // 보상 계산
    let reward = (stake_amount as f64 * reward_rate * periods) as u64;
    ```
    
2. vault account로부터 유저 ATA로 보상을 전송한다.
    
    ```rust
            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.token_vault_account.to_account_info(),
                        to: ctx.accounts.user_token_account.to_account_info(),
                        authority: ctx.accounts.token_vault_account.to_account_info(),
                    },
                    signer,
                ),
                reward,
            )?;
    ```
    
3. stake account에서 ATA로 스테이킹한 수량을 전송한다.
    
    ```
            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.stake_account.to_account_info(),
                        to: ctx.accounts.user_token_account.to_account_info(),
                        authority: ctx.accounts.stake_account.to_account_info(),
                    },
                    signer,
                ),
                stake_amount,
            )?;
    ```
