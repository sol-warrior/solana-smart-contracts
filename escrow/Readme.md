# 🏦 Solana Escrow Smart Contract

A trustless token swap protocol built on Solana that enables secure peer-to-peer token exchanges without requiring intermediaries or custodians.

![Escrow Architecture Diagram](escrow.png)

## 🎯 Overview

The Escrow program allows two parties to exchange tokens in a completely trustless manner. One party (the **Maker**) creates an escrow offer by depositing their tokens and specifying what they want in return. Another party (the **Taker**) can then fulfill this offer by providing the requested tokens, triggering an atomic swap.

## 📊 Program Details

- **Program ID**: `KMYaZW7KUS6atp5F645vZ5ynuR6aZ7TueiTtTCRes6e`
- **IDL Account**: `7PeFYqJkexyRgbGM4gyCbCatKqN3Th3UjhSZ1B5eMVUn`
- **Deployed Signature**: `42x8yZCB6ufEpCDq5YAuKF1gozj8Kj4TudSDy92aTSiXpLLT8iQLDnceG3wz4zTceAHx8AC8AVApMwkt9qAHZUxc`
- **Security Metadata**: `7bnqA5qg94DCKuiAAzzJVit88iByuwJfpsUvc887PekW`
- **Framework**: Anchor 0.32.1
- **Language**: Rust
- **Blockchain**: Solana
- **Token Standard**: SPL Token (Token-2022 compatible)

## 👥 User Stories

### 🤝 As a Token Trader

**Scenario: Secure Token Swap**

- **Given** I have 1000 USDC that I want to exchange for BONK tokens
- **When** I create an escrow offer specifying `10,000 USDC for 15,000 BONK`
- **Then** my USDC is safely locked in a program-controlled vault
- **And** anyone can see and fulfill my offer instantly

### 🛡️ As a Risk-Averse User

**Scenario: Trustless Exchange**

- **Given** I found an escrow offer that matches my needs
- **When** I provide the exact tokens requested
- **Then** I receive the offered tokens immediately
- **And** no party can cheat or back out of the deal

### 🔄 As an Offer Creator

**Scenario: Refund Option**

- **Given** my escrow offer hasn't been taken
- **When** I want my tokens back
- **Then** I can refund my deposited tokens anytime
- **And** the escrow is cleanly closed

## 🏗️ Architecture

### Core Components

#### **Escrow Account (PDA)**

- **Purpose**: Stores trade parameters and acts as the trade's central authority
- **Seeds**: `["escrow", maker_pubkey, seed]`
- **Data**:
  - `seed`: Unique identifier for the escrow
  - `maker`: Public key of the offer creator
  - `mint_m`: Token mint address the maker is offering
  - `mint_n`: Token mint address the maker wants in return
  - `token_mint_n_expected`: Amount of token N required to fulfill the offer

#### **Vault Account (ATA)**

- **Purpose**: Securely holds the maker's deposited tokens
- **Authority**: Escrow PDA (program-controlled)
- **Token Mint**: Same as maker's offered token

### Program Instructions

#### **1. Maker Instruction** 📝

Creates a new escrow offer and deposits tokens.

**Parameters:**

- `seed` (u64): Unique identifier for this escrow
- `token_mint_n_expected` (u64): Amount of token N wanted
- `amount` (u64): Amount of token M to deposit

**Actions:**

- Creates escrow PDA account
- Creates vault ATA for token storage
- Transfers maker's tokens to vault
- Initializes escrow state

#### **2. Taker Instruction** ✅

Fulfills an existing escrow offer.

**Parameters:** None (reads requirements from escrow account)

**Actions:**

- Validates taker has sufficient tokens
- Transfers taker's tokens to maker
- Transfers vault tokens to taker
- Closes escrow and vault accounts

#### **3. Refund Instruction** ↩️

Allows maker to reclaim their tokens if offer expires.

**Parameters:** None

**Actions:**

- Validates caller is the original maker
- Transfers vault tokens back to maker
- Closes escrow and vault accounts

## 🔧 Technical Implementation

### Security Features

- **Program-Derived Addresses (PDAs)**: Deterministic, program-controlled accounts
- **Authority Checks**: Only authorized parties can perform actions
- **Atomic Transactions**: All operations succeed or fail together
- **Token Interface**: Compatible with all SPL tokens

### Error Handling

```rust
#[error_code]
pub enum EscrowError {
    #[msg("Invalid amount")] InvalidAmount,
    #[msg("Invalid maker")] InvalidMaker,
    #[msg("Invalid mint m")] InvalidMintM,
    #[msg("Invalid mint n")] InvalidMintN,
}
```

### Events

```rust
#[event]
pub struct EscrowReady {
    pub escrow: Pubkey,
    pub maker: Pubkey,
    pub taker: Pubkey,
}
```

## 🚀 Quick Start

### Prerequisites

- Node.js 16+
- Yarn package manager
- Solana CLI tools
- Rust toolchain

### Installation

```bash
# Clone the repository
git clone https://github.com/sol-warrior/solana-smart-contracts
cd solana-smart-contracts/escrow

# Install dependencies
yarn install

# Build the program
anchor build

# Run tests
anchor test
```

### Deployment

```bash
# Deploy to localnet
anchor deploy

# Deploy to devnet/mainnet
anchor deploy --provider.cluster devnet
```

## 📋 Usage Examples

### Creating an Escrow Offer

```typescript
// Example: Offer 1000 USDC for 50,000 BONK
const seed = new anchor.BN(1);
const offeredAmount = new anchor.BN(1000 * 10 ** 6); // 1000 USDC (6 decimals)
const expectedAmount = new anchor.BN(50000 * 10 ** 6); // 50,000 BONK (6 decimals)

await program.methods
  .maker(seed, expectedAmount, offeredAmount)
  .accounts({
    maker: maker.publicKey,
    mintM: usdcMint,
    mintN: bonkMint,
    // ... other accounts
  })
  .rpc();
```

### Fulfilling an Offer

```typescript
// Taker fulfills the escrow
await program.methods
  .taker()
  .accounts({
    taker: taker.publicKey,
    escrow: escrowPda,
    // ... other accounts
  })
  .rpc();
```

### Refunding an Offer

```typescript
// Maker refunds their tokens
await program.methods
  .refund()
  .accounts({
    maker: maker.publicKey,
    escrow: escrowPda,
    // ... other accounts
  })
  .rpc();
```

## 🧪 Testing

The project includes comprehensive tests using `LiteSVM` for fast, local testing:

```bash
# Run all tests
yarn test

# Run specific test suite
yarn ts-mocha tests/litesvm.test.ts
```

**Test Coverage:**

- ✅ Escrow creation and token deposit
- ✅ Successful token swap fulfillment
- ✅ Refund functionality
- ✅ Account validation and security checks

## 🔐 Security Considerations

1. **Access Control**: Only authorized users can execute specific instructions
2. **Amount Validation**: Prevents zero-amount and invalid trades
3. **PDA Security**: Program-derived addresses prevent unauthorized access
4. **Atomic Operations**: All token transfers happen in single transactions
5. **Account Validation**: Comprehensive checks on all account relationships

## 🤝 Contributing

Contributions are welcome and appreciated.

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

---

## 📄 License

This project is licensed under the **ISC License**, a permissive open-source license similar to MIT.

See the [LICENSE](LICENSE) file for full details.

---

## 📞 Support

If you have questions or issues:

- Open an issue on GitHub
- Review the test files for usage examples
- Refer to the Anchor documentation for Solana development

---

## 🌐 Connect with Me

- GitHub: https://github.com/sol-warrior
- X (Twitter): https://x.com/warriorofsol

Built with ❤️ using **Anchor Framework** on **Solana**.
