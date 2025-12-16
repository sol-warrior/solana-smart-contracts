### Solana Vault (Anchor + LiteSVM + Next.js)

This project is a **Solana lamport vault** built with the **Anchor framework** and a modern **Next.js / React** front‑end. Each connected wallet can initialize its own program‑derived "vault" accounts, deposit lamports, withdraw them later, and finally close the vault.

The front‑end uses `@solana/kit` style patterns to talk to the on‑chain Anchor program via its IDL, providing a clean, wallet‑first UX.

---

### On‑chain program

- **Program name**: `vault`
- **Cluster (default)**: `devnet` (configured in `Anchor.toml`)
- **Program ID**: `DJktBCt1vV8jdYmyxqnH8oNVa5PMLWgkc7WuT4KB1Q8o`
- **IDL Account**: `ADrfF68UrJUCQFdJ3ZGf92NEjAXgUtnzX9qpKxdopgBP`
- **Security metadata address**: `D9rgGKnUojaWqh19DuAFW22gbuRYNqAfXjsBGFkEtjp6`
- **Initial deploy signature**: `38t31hMWHmwmD4dYqyJuMYFePVZgEq1zqBumQBBqgVGDuzNzBfcri4Biam4CZDCMS194dzvBWZYVCnDqsbx4TMt`

#### What is the metadata address?

Think of the **metadata address** as a "business card" stored on the blockchain. It contains public information about the program like:
- The program's name and description
- Project website/GitHub links
- Contact information for security reports
- Other verification details

This metadata helps users verify that a program is legitimate and provides a way to contact developers if security issues are found. It's created automatically by Anchor when you publish security information about your program.

#### View on Solana Explorer

You can inspect the program and its metadata directly on Solana Explorer:

- **Program account**: [View Program](https://explorer.solana.com/address/DJktBCt1vV8jdYmyxqnH8oNVa5PMLWgkc7WuT4KB1Q8o?cluster=devnet)
- **Security metadata**: [View Metadata](https://explorer.solana.com/address/D9rgGKnUojaWqh19DuAFW22gbuRYNqAfXjsBGFkEtjp6?cluster=devnet)
- **Deploy transaction**: [View Transaction](https://explorer.solana.com/tx/38t31hMWHmwmD4dYqyJuMYFePVZgEq1zqBumQBBqgVGDuzNzBfcri4Biam4CZDCMS194dzvBWZYVCnDqsbx4TMt?cluster=devnet)

The program exposes four main instructions (see `target/idl/vault.json`):

- **initialize**: creates the user’s vault PDA and the associated lamports PDA.
- **deposit(amount: u64)**: moves lamports from the user’s wallet into their lamport vault PDA and updates total deposited.
- **withdraw(amount: u64)**: withdraws lamports from the vault PDA back to the user, enforcing balance and owner checks.
- **close**: closes the user’s vault and lamport accounts, returning remaining lamports to the owner.

PDAs are derived using fixed seeds and the user public key, for example:

- **Vault PDA**: seed `"vault"` + user pubkey
- **Lamports PDA**: seed `"user_lamports"` + user pubkey + vault PDA

Common error codes are also exported via the IDL (e.g. `VaultAlreadyInitialized`, `NotVaultOwner`, `InvalidAmount`, `InsufficientFunds`), and are surfaced in the UI.

---

### Front‑end (Next.js app)

The `app/` directory contains a **Next.js (App Router) UI** that allows you to:

- **Connect a wallet** using the Wallet Standard UI.
- **Initialize your vault** and create the PDAs.
- **Deposit / withdraw lamports** via simple input controls.
- **Close your vault** when you’re done.

Key components:

- **`app/page.tsx`**: top‑level page layout and wiring between wallet and vault actions.
- **`components/wallet/WalletSection.tsx`**: wallet connect / account selection.
- **`components/vault/VaultActions.tsx`**: calls `initialize`, `deposit`, `withdraw`, and `close` through a small client built from the Anchor IDL.

The UI is styled as a dark, dashboard‑style panel and displays:

- current **vault PDAs** (shortened for readability),
- **total deposited lamports**, and
- any relevant **program errors**.

---

### Tech stack

- **Solana program**: Rust + **Anchor**
- **Client SDKs**: `@coral-xyz/anchor`, `@solana/web3.js`
- **Testing / tooling**: `ts-mocha`, `chai`, `litesvm`
- **Front‑end**: **Next.js / React**, Tailwind‑style utility classes

---

### Getting started (local development)

#### Prerequisites

- **Node.js** (LTS) and **yarn**
- **Rust** + **Cargo**
- **Solana CLI** (`solana`)
- **Anchor CLI** (`anchor`)

#### 1. Install dependencies

```bash
yarn install
```

#### 2. Configure Solana

Make sure your Solana CLI is pointing to the same cluster as `Anchor.toml` (currently `devnet`):

```bash
solana config set --url devnet
```

Ensure your wallet keypair is available at `~/.config/solana/id.json` or update the `wallet` path in `Anchor.toml`.

#### 3. Build and deploy the program

If you want to re‑deploy from source:

```bash
anchor build
anchor deploy
```

The deploy step will output a **Program Id** and **Signature**; this repo currently uses:

- Program Id: `DJktBCt1vV8jdYmyxqnH8oNVa5PMLWgkc7WuT4KB1Q8o`
- Deploy signature: `38t31hMWHmwmD4dYqyJuMYFePVZgEq1zqBumQBBqgVGDuzNzBfcri4Biam4CZDCMS194dzvBWZYVCnDqsbx4TMt`

If you redeploy with a different program id, also update:

- `Anchor.toml` under `[programs.<cluster>].vault`
- any client config that references the old address.

#### 4. Run tests (optional)

```bash
yarn test
# or, using Anchor’s script
anchor test
```

#### 5. Run the front‑end

From the `app/` directory (or root, depending on your Next.js setup), start the dev server, for example:

```bash
cd app
npm install   # or yarn
npm run dev   # or yarn dev
```

Open the printed URL in your browser (typically `http://localhost:3000`) and:

1. Connect your Solana wallet.
2. Initialize your vault.
3. Deposit some lamports.
4. Withdraw or close the vault to reclaim funds.

---

### Notes

- This project is intended as an **educational example** of integrating an Anchor program with a modern React / Next.js front‑end.
- Always be careful when deploying to **mainnet‑beta**; review and audit the program code before managing real funds.
