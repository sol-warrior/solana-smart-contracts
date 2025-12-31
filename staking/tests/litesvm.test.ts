import { LiteSVM, TransactionMetadata } from "litesvm";
import anchor from "@coral-xyz/anchor";
import { assert, expect } from "chai";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";
import { getAssociatedTokenAddressSync, AccountLayout, ACCOUNT_SIZE, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, MintLayout, MINT_SIZE } from "@solana/spl-token";
import Idl from "../target/idl/staking.json" with { type: "json" };


describe("LiteSVM: Staking", () => {
    const svm = new LiteSVM();
    const programId = new PublicKey(Idl.address);
    const coder = new anchor.BorshCoder(Idl as anchor.Idl);

    const poolCreator = Keypair.generate();
    const staker = Keypair.generate();
    svm.airdrop(poolCreator.publicKey, BigInt(10 * LAMPORTS_PER_SOL));
    svm.airdrop(staker.publicKey, BigInt(5 * LAMPORTS_PER_SOL));

    const programPath = new URL("../target/deploy/staking.so", import.meta.url).pathname;
    svm.addProgramFromFile(programId, programPath);

    const usdcMint = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");


    const [poolPda] = PublicKey.findProgramAddressSync([
        Buffer.from("pool"),
        poolCreator.publicKey.toBuffer(),
    ], programId);

    const [userStakePda] = PublicKey.findProgramAddressSync([
        Buffer.from("user-stake"),
        staker.publicKey.toBuffer(),
        poolPda.toBuffer()
    ], programId);

    const poolVault = getAssociatedTokenAddressSync(usdcMint, poolPda, true);
    const userAta = getAssociatedTokenAddressSync(usdcMint, staker.publicKey, true); //USDC token 

    const stakerHaveUsdc = BigInt(1_000_000_000_000);

    const stakedTokenAmount = new anchor.BN(5000 * 10 ** 6); //USDC
    const unstakeSomeTokenAmount = new anchor.BN(3000 * 10 ** 6); //USDC

    before("Initialized MINT token", () => {
        const usdcMintAuthority = PublicKey.unique();
        const usdcMintData = Buffer.alloc(MINT_SIZE);

        MintLayout.encode(
            {
                mintAuthorityOption: 1,
                mintAuthority: usdcMintAuthority,
                supply: BigInt(0),
                decimals: 6,
                isInitialized: true,
                freezeAuthorityOption: 0,
                freezeAuthority: PublicKey.default,
            },
            usdcMintData
        );

        svm.setAccount(usdcMint, {
            lamports: 1_000_000_000,
            data: usdcMintData,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
        });

        const usdcMintAcct = svm.getAccount(usdcMint);
        const uMintData = usdcMintAcct?.data;
        const usdcDecoded = MintLayout.decode(uMintData);

        expect(usdcMintAcct).to.not.be.null;
        expect(uMintData).to.not.be.undefined;
        expect(usdcDecoded.isInitialized).to.equal(true);
        expect(usdcDecoded.decimals).to.equal(6);
    })

    before("Initialized ATA (Associated Token Account)", () => {
        const stakerAccData = Buffer.alloc(ACCOUNT_SIZE);

        AccountLayout.encode(
            {
                mint: usdcMint,
                owner: staker.publicKey,
                amount: stakerHaveUsdc,
                delegateOption: 0,
                delegate: PublicKey.default,
                delegatedAmount: BigInt(0),
                state: 1,
                isNativeOption: 0,
                isNative: BigInt(0),
                closeAuthorityOption: 0,
                closeAuthority: PublicKey.default,
            },
            stakerAccData,
        );

        svm.setAccount(userAta, {
            lamports: 1_000_000_000,
            data: stakerAccData,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
        });

        const usdcMintAcct = svm.getAccount(usdcMint);
        const uMintData = usdcMintAcct?.data;
        const usdcDecoded = MintLayout.decode(uMintData);

        expect(usdcMintAcct).to.not.be.null;
        expect(uMintData).to.not.be.undefined;
        expect(usdcDecoded.isInitialized).to.equal(true);
        expect(usdcDecoded.decimals).to.equal(6);
    })

    it("Initialize Pool", async () => {
        const data = coder.instruction.encode("initialize_pool", {});

        const ix = new TransactionInstruction({
            keys: [
                { pubkey: poolCreator.publicKey, isSigner: true, isWritable: true },
                { pubkey: poolPda, isSigner: false, isWritable: true },
                { pubkey: usdcMint, isSigner: false, isWritable: false },
                { pubkey: poolVault, isSigner: false, isWritable: true },
                { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
            ],
            programId,
            data
        })

        const tx = new Transaction().add(ix);
        tx.recentBlockhash = svm.latestBlockhash();
        tx.feePayer = poolCreator.publicKey;
        tx.sign(poolCreator);

        svm.sendTransaction(tx);

        const poolAccInfo = svm.getAccount(poolPda);
        const poolAcc = coder.accounts.decode("Pool", Buffer.from(poolAccInfo.data));

        assert.equal(usdcMint.toString(), poolAcc.mint.toString());
        assert.equal(poolVault.toString(), poolAcc.vault.toString());
        assert.equal(poolAcc.reward_rate, 1);
        assert.equal(poolAcc.total_staked, 0);
    });

    it("Stake USDC Token", async () => {
        const ixArgs = {
            amount: stakedTokenAmount
        }
        const data = coder.instruction.encode("stake", ixArgs);

        const ix = new TransactionInstruction({
            keys: [
                { pubkey: staker.publicKey, isSigner: true, isWritable: true },
                { pubkey: poolCreator.publicKey, isSigner: false, isWritable: true },
                { pubkey: poolPda, isSigner: false, isWritable: true },
                { pubkey: usdcMint, isSigner: false, isWritable: false },
                { pubkey: poolVault, isSigner: false, isWritable: true },
                { pubkey: userStakePda, isSigner: false, isWritable: true },
                { pubkey: userAta, isSigner: false, isWritable: true },
                { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
            ],
            programId,
            data
        })

        const tx = new Transaction().add(ix);
        tx.recentBlockhash = svm.latestBlockhash();
        tx.feePayer = staker.publicKey;
        tx.sign(staker);

        svm.sendTransaction(tx);

        const stakeAccInfo = svm.getAccount(userStakePda);
        const stakeAcc = coder.accounts.decode("UserStake", Buffer.from(stakeAccInfo.data));
        const poolAccInfo = svm.getAccount(poolPda);
        const poolAcc = coder.accounts.decode("Pool", Buffer.from(poolAccInfo.data));
        const poolVaultInfo = svm.getAccount(poolVault);
        const poolVaultAcc = AccountLayout.decode(poolVaultInfo.data);

        assert.equal(Number(poolVaultAcc.amount), stakedTokenAmount.toNumber(), "Staked token at pool vault account");
        assert.equal(usdcMint.toString(), poolAcc.mint.toString(), "USDC address matches with pool mint address");
        assert.equal(poolVault.toString(), poolAcc.vault.toString());
        assert.equal(userAta.toString(), stakeAcc.user_vault_ata.toString());
        assert.equal(Number(stakeAcc.points), Number(0));
        assert.equal(stakeAcc.amount.toNumber(), stakedTokenAmount.toNumber());
        assert.equal(poolAcc.total_staked.toNumber(), stakedTokenAmount.toNumber());
    });


    it("UnStake some amount of USDC Token", async () => {
        const ixArgs = {
            amount: unstakeSomeTokenAmount
        }
        const data = coder.instruction.encode("unstake", ixArgs);

        const ix = new TransactionInstruction({
            keys: [
                { pubkey: staker.publicKey, isSigner: true, isWritable: true },
                { pubkey: poolCreator.publicKey, isSigner: false, isWritable: true },
                { pubkey: poolPda, isSigner: false, isWritable: true },
                { pubkey: usdcMint, isSigner: false, isWritable: false },
                { pubkey: poolVault, isSigner: false, isWritable: true },
                { pubkey: userStakePda, isSigner: false, isWritable: true },
                { pubkey: userAta, isSigner: false, isWritable: true },
                { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
            ],
            programId,
            data
        })

        const tx = new Transaction().add(ix);
        tx.recentBlockhash = svm.latestBlockhash();
        tx.feePayer = staker.publicKey;
        tx.sign(staker);

        svm.sendTransaction(tx);

        const stakeAccInfo = svm.getAccount(userStakePda);
        const stakeAcc = coder.accounts.decode("UserStake", Buffer.from(stakeAccInfo.data));
        const poolAccInfo = svm.getAccount(poolPda);
        const poolAcc = coder.accounts.decode("Pool", Buffer.from(poolAccInfo.data));
        const poolVaultInfo = svm.getAccount(poolVault);
        const poolVaultAcc = AccountLayout.decode(poolVaultInfo.data);


        assert.equal(Number(poolVaultAcc.amount), Number(stakedTokenAmount) - Number(unstakeSomeTokenAmount), "Unstaked token at pool vault account");
        assert.equal(usdcMint.toString(), poolAcc.mint.toString(), "USDC address matches with pool mint address");
        assert.equal(poolVault.toString(), poolAcc.vault.toString(), "Pool account created by creator");
        assert.equal(userAta.toString(), stakeAcc.user_vault_ata.toString(), "User USDC token account must match with stake's account");
        assert.equal(Number(stakeAcc.points), Number(0));
        assert.equal(stakeAcc.amount.toNumber(), Number(stakedTokenAmount) - Number(unstakeSomeTokenAmount), "Stake token account must decrease after unstake");
        assert.equal(poolAcc.total_staked.toNumber(), Number(stakedTokenAmount) - Number(unstakeSomeTokenAmount), " Pool token account must descrease the user's stake token amount");
    });

    it("Get points for stake token", () => {
        const ix = new TransactionInstruction({
            keys: [
                { pubkey: staker.publicKey, isSigner: true, isWritable: false },
                { pubkey: poolPda, isSigner: false, isWritable: false },
                { pubkey: userStakePda, isSigner: false, isWritable: false }
            ],
            programId,
            data: Buffer.from([2]) //get_points
        });

        const tx = new Transaction().add(ix);
        tx.feePayer = staker.publicKey;
        tx.recentBlockhash = svm.latestBlockhash();
        tx.sign(staker);

        const res = svm.sendTransaction(tx);

        const resData = (res as TransactionMetadata).returnData();
        // console.log(resData.toString())

        const buffData = Buffer.from(resData.data())
        const getPointData = buffData.readBigUInt64LE(0)
        // console.log(getPointData);

        expect(Number(getPointData) === 0)

    })

    it("UnStakeAll USDC Token", async () => {
        const ix = new TransactionInstruction({
            keys: [
                { pubkey: staker.publicKey, isSigner: true, isWritable: true },
                { pubkey: poolCreator.publicKey, isSigner: false, isWritable: true },
                { pubkey: poolPda, isSigner: false, isWritable: true },
                { pubkey: usdcMint, isSigner: false, isWritable: false },
                { pubkey: poolVault, isSigner: false, isWritable: true },
                { pubkey: userStakePda, isSigner: false, isWritable: true },
                { pubkey: userAta, isSigner: false, isWritable: true },
                { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
            ],
            programId,

            // The staking program expects the first bytes of the data buffer to match the discriminator for the instruction to the invoke.
            // In the IDL, "unstake_all" has a discriminator of [5], so we must send Buffer.from([5]).
            // Buffer.from("unstake_all") is a UTF-8 string, which will not match the expected single-byte discriminator and the program will reject the transaction.
            data: Buffer.from([5])//unstake_all
        })

        const tx = new Transaction().add(ix);
        tx.recentBlockhash = svm.latestBlockhash();
        tx.feePayer = staker.publicKey;
        tx.sign(staker);

        svm.sendTransaction(tx);

        const poolAccInfo = svm.getAccount(poolPda);
        const poolAcc = coder.accounts.decode("Pool", Buffer.from(poolAccInfo.data));
        assert.equal(poolAcc.total_staked.toNumber(), Number(stakedTokenAmount) - Number(stakedTokenAmount), " Pool token account state must descrease after withdraw all token");

        const poolVaultInfo = svm.getAccount(poolVault);//usdc 
        const poolVaultAcc = AccountLayout.decode(poolVaultInfo.data);
        assert.equal(Number(poolVaultAcc.amount), Number(stakedTokenAmount) - Number(stakedTokenAmount), "Pool vault must amount decrease ");

        const userStakeAcc = svm.getAccount(userStakePda);
        assert.isNull(userStakeAcc, "UserStake account should be closed after unstake all token");

    });


})