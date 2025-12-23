import { LiteSVM } from "litesvm";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";
import anchor from "@coral-xyz/anchor";
import { getAssociatedTokenAddressSync, AccountLayout, ACCOUNT_SIZE, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, MintLayout, MINT_SIZE, } from "@solana/spl-token";
import Idl from "../target/idl/escrow.json" with {type: "json"};
import { assert, expect } from "chai";

describe("LiteSVM: Escrow", () => {

    const svm = new LiteSVM();
    const programId = new PublicKey(Idl.address);
    const coder = new anchor.BorshCoder(Idl as anchor.Idl);

    const maker = Keypair.generate();
    const taker = Keypair.generate();
    svm.airdrop(maker.publicKey, BigInt(10 * LAMPORTS_PER_SOL));
    svm.airdrop(taker.publicKey, BigInt(5 * LAMPORTS_PER_SOL));

    const programPath = new URL("../target/deploy/escrow.so", import.meta.url).pathname;
    svm.addProgramFromFile(programId, programPath);

    const usdcMint = new PublicKey(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    );
    const bonkMint = new PublicKey(
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"
    );

    const [escrowPda] = PublicKey.findProgramAddressSync([
        Buffer.from("escrow"), maker.publicKey.toBuffer(),
        new anchor.BN(1).toArrayLike(Buffer, "le", 8) //To match Rust's `seed.to_le_bytes().as_ref()`
    ], programId);

    const makerAtaM = getAssociatedTokenAddressSync(usdcMint, maker.publicKey, true);
    const takerAtaM = getAssociatedTokenAddressSync(usdcMint, taker.publicKey, true);
    const takerAtaN = getAssociatedTokenAddressSync(bonkMint, taker.publicKey, true);
    const makerAtaN = getAssociatedTokenAddressSync(bonkMint, maker.publicKey, true);
    const vaultAtaM = getAssociatedTokenAddressSync(usdcMint, escrowPda, true);
    const MakerHaveUsdc = BigInt(1_000_000_000_000);
    const TakerHaveBonk = BigInt(50_000_000_000);
    const makerOfferedAmount = new anchor.BN(3000 * 10 ** 6);  //USDC
    const makerExpectedAmount = new anchor.BN(6000 * 10 ** 6); //BONK

    before("Initialized MINT token", () => {

        const usdcMintAuthority = PublicKey.unique();
        const bonkMintAuthority = PublicKey.unique();

        //  Allocate space for Mint account
        const usdcMintData = Buffer.alloc(MINT_SIZE);
        const bonkMintData = Buffer.alloc(MINT_SIZE);

        // Encode a VALID SPL Mint (USDC)
        MintLayout.encode(
            {
                mintAuthorityOption: 1,  // authority exists
                mintAuthority: usdcMintAuthority,
                supply: BigInt(0),
                decimals: 6,
                isInitialized: true,
                freezeAuthorityOption: 0,
                freezeAuthority: PublicKey.default,
            },
            usdcMintData
        );

        //  Encode a VALID SPL Mint (BONK)
        MintLayout.encode(
            {
                mintAuthorityOption: 1,
                mintAuthority: bonkMintAuthority,
                supply: BigInt(0),
                decimals: 6,
                isInitialized: true,
                freezeAuthorityOption: 0,
                freezeAuthority: PublicKey.default,
            },
            bonkMintData
        );


        svm.setAccount(usdcMint, {
            lamports: 1_000_000_000,  // rent-exempt enough
            data: usdcMintData,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
        });

        svm.setAccount(bonkMint, {
            lamports: 1_000_000_000,
            data: bonkMintData,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
        });

        // Check BONK mint
        const bonkMintAcct = svm.getAccount(bonkMint);
        const bMintData = bonkMintAcct?.data;
        const bonkDecoded = MintLayout.decode(bMintData);

        expect(bonkMintAcct).to.not.be.null;
        expect(bMintData).to.not.be.undefined;
        expect(bonkDecoded.isInitialized).to.equal(true);
        expect(bonkDecoded.decimals).to.equal(6);

        // Check USDC mint
        const usdcMintAcct = svm.getAccount(usdcMint);
        const uMintData = usdcMintAcct?.data;
        const usdcDecoded = MintLayout.decode(uMintData);

        expect(usdcMintAcct).to.not.be.null;
        expect(uMintData).to.not.be.undefined;
        expect(usdcDecoded.isInitialized).to.equal(true);
        expect(usdcDecoded.decimals).to.equal(6);

    })

    before("Initialized ATA (Associated Token Account)", () => {
        const makerAccData = Buffer.alloc(ACCOUNT_SIZE);
        const takerAccData = Buffer.alloc(ACCOUNT_SIZE);

        // Initializing Maker's USDC ATA with balance
        AccountLayout.encode(
            {
                mint: usdcMint,
                owner: maker.publicKey,
                amount: MakerHaveUsdc,
                delegateOption: 0,
                delegate: PublicKey.default,
                delegatedAmount: BigInt(0),
                state: 1,
                isNativeOption: 0,
                isNative: BigInt(0),
                closeAuthorityOption: 0,
                closeAuthority: PublicKey.default,
            },
            makerAccData,
        );

        svm.setAccount(makerAtaM, {
            lamports: 1_000_000_000,
            data: makerAccData,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
        });


        // For Taker
        AccountLayout.encode(
            {
                mint: bonkMint,
                owner: taker.publicKey,
                amount: TakerHaveBonk,
                delegateOption: 0,
                delegate: PublicKey.default,
                delegatedAmount: BigInt(0),
                state: 1,
                isNativeOption: 0,
                isNative: BigInt(0),
                closeAuthorityOption: 0,
                closeAuthority: PublicKey.default,
            },
            takerAccData,
        );

        svm.setAccount(takerAtaN, {
            lamports: 1_000_000_000,
            data: takerAccData,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
        });

        const makerRawAccount = svm.getAccount(makerAtaM);
        const makerRawAccountData = makerRawAccount?.data;
        const makerDecoded = AccountLayout.decode(makerRawAccountData);
        const takerRawAccount = svm.getAccount(takerAtaN);
        const takerRawAccountData = takerRawAccount?.data;
        const takerDecoded = AccountLayout.decode(takerRawAccountData);

        expect(makerRawAccount, "Maker's ATA should exist after mint/ATA setup").to.not.be.null;
        expect(makerDecoded.amount, "Maker's ATA should hold the correct initial USDC amount").to.eql(MakerHaveUsdc);
        expect(takerRawAccount, "Taker's ATA should exist after mint/ATA setup").to.not.be.null;
        expect(takerDecoded.amount, "Taker's ATA should hold the correct initial USDC amount").to.eql(TakerHaveBonk);
    });

    it("Maker: Creates an escrow and deposits funds", async () => {
        const ixArgs = {
            amount: makerOfferedAmount, //3000 USDC giving
            token_mint_n_expected: makerExpectedAmount, // 6000 BONK expecting
            seed: new anchor.BN(1)

        }
        const data = coder.instruction.encode("maker", ixArgs);

        const ix = new TransactionInstruction({
            keys: [
                { pubkey: maker.publicKey, isSigner: true, isWritable: true },
                { pubkey: escrowPda, isSigner: false, isWritable: true },
                { pubkey: usdcMint, isSigner: false, isWritable: false },
                { pubkey: bonkMint, isSigner: false, isWritable: false },
                { pubkey: makerAtaM, isSigner: false, isWritable: true },
                { pubkey: vaultAtaM, isSigner: false, isWritable: true },
                { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
            ],
            programId,
            data
        })

        const tx = new Transaction().add(ix);
        tx.feePayer = maker.publicKey;
        tx.recentBlockhash = svm.latestBlockhash();
        tx.sign(maker);
        svm.sendTransaction(tx);

        const makerAtaMAcc = svm.getAccount(makerAtaM);
        const makerAtaMAccInfo = AccountLayout.decode(makerAtaMAcc.data);
        const vaultAtaMAcc = svm.getAccount(vaultAtaM);
        const vaultAtaMAccInfo = AccountLayout.decode(vaultAtaMAcc.data);
        const escrowAccInfo = svm.getAccount(escrowPda);
        const escrowAcc = coder.accounts.decode("Escrow", Buffer.from(escrowAccInfo.data));


        // This test ensures that after calling the "maker" instruction,
        // 1. The escrow account contains the correct USDC mint as mint_m.
        // 2. The escrow account contains the correct BONK mint as mint_n.
        // 3. The escrow's seed matches what we passed.
        // 4. The escrow's expected amount for mint_n matches what we passed.
        // 5. The correct amount of USDC was deposited into the vault account.
        // 6. The payer's ATA for USDC was debited by the deposited amount, leaving the correct remaining balance.
        assert.equal(usdcMint.toString(), escrowAcc.mint_m.toString(), "Escrow mint_m should be USDC mint public key");
        assert.equal(bonkMint.toString(), escrowAcc.mint_n.toString(), "Escrow mint_n should be BONK mint public key");
        assert.equal(ixArgs.seed.toNumber(), escrowAcc.seed.toNumber(), "Escrow seed should match what was provided");
        assert.equal(ixArgs.token_mint_n_expected.toNumber(), escrowAcc.token_mint_n_expected.toNumber(), "Escrow token_mint_n_expected should match requested");
        assert.equal(ixArgs.amount.toNumber(), Number(vaultAtaMAccInfo.amount), "Vault account should contain exactly deposited USDC");
        assert.equal(Number(MakerHaveUsdc) - Number(ixArgs.amount), Number(makerAtaMAccInfo.amount), "Maker's ATA should contain remaining USDC after deposit");
    })

    it.skip("Refund: Close an escrow and withdraws funds", async () => {
        const data = coder.instruction.encode("refund", {});

        const ix = new TransactionInstruction({
            keys: [
                { pubkey: maker.publicKey, isSigner: true, isWritable: true },
                { pubkey: escrowPda, isSigner: false, isWritable: true },
                { pubkey: usdcMint, isSigner: false, isWritable: false },
                { pubkey: bonkMint, isSigner: false, isWritable: false },
                { pubkey: makerAtaM, isSigner: false, isWritable: true },
                { pubkey: vaultAtaM, isSigner: false, isWritable: true },
                { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
            ],
            programId,
            data
        })

        const tx = new Transaction().add(ix);
        tx.feePayer = maker.publicKey;
        tx.recentBlockhash = svm.latestBlockhash();
        tx.sign(maker);
        svm.sendTransaction(tx);

        const escrowAccount = svm.getAccount(escrowPda);
        assert.isNull(escrowAccount, "Escrow account should be closed after refund");

        const vaultAccount = svm.getAccount(vaultAtaM);
        assert.isNull(vaultAccount, "Vault ATA should be closed after refund");

        const makerAtaMAccAfter = svm.getAccount(makerAtaM);
        const makerAtaMAccInfoAfter = AccountLayout.decode(makerAtaMAccAfter.data);
        assert.equal(
            Number(makerAtaMAccInfoAfter.amount),
            Number(MakerHaveUsdc),
            "Maker's ATA should be fully refunded"
        );
    })

    it("Taker: Deposits expected funds and withdraws offered funds", async () => {
        const data = coder.instruction.encode("taker", {});

        const ix = new TransactionInstruction({
            keys: [
                { pubkey: taker.publicKey, isSigner: true, isWritable: true },
                { pubkey: maker.publicKey, isSigner: false, isWritable: true },
                { pubkey: escrowPda, isSigner: false, isWritable: true },
                { pubkey: usdcMint, isSigner: false, isWritable: false },
                { pubkey: bonkMint, isSigner: false, isWritable: false },
                { pubkey: vaultAtaM, isSigner: false, isWritable: true },
                { pubkey: takerAtaM, isSigner: false, isWritable: true },
                { pubkey: takerAtaN, isSigner: false, isWritable: true }, ,
                { pubkey: makerAtaN, isSigner: false, isWritable: true },
                { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
                { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
            ],
            programId,
            data
        })

        const tx = new Transaction().add(ix);
        tx.feePayer = taker.publicKey;
        tx.recentBlockhash = svm.latestBlockhash();
        tx.sign(taker);
        svm.sendTransaction(tx);


        const makerAtaMAcc = svm.getAccount(makerAtaM);
        const makerAtaMAccInfo = AccountLayout.decode(makerAtaMAcc.data);
        const takerAtaMAcc = svm.getAccount(takerAtaN);
        const takerAtaMAccInfo = AccountLayout.decode(takerAtaMAcc.data);

        const escrowAccInfo = svm.getAccount(escrowPda);
        const vaultAtaMAcc = svm.getAccount(vaultAtaM);

        assert.isNull(vaultAtaMAcc, "Vault ATA should be closed after taker's withdrawal");
        assert.isNull(escrowAccInfo, "Escrow account should be closed after taker's withdrawal");
        assert.equal(Number(MakerHaveUsdc) - Number(makerOfferedAmount), Number(makerAtaMAccInfo.amount), "Maker's ATA should contain remaining USDC after deposit");
        assert.equal(Number(TakerHaveBonk) - Number(makerExpectedAmount), Number(takerAtaMAccInfo.amount), "Taker's ATA should contain remaining BONK after deposit");
    })
});

