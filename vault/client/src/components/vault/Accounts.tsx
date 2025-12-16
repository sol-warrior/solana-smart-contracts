"use client";
import {
  getUserVaultLamportsPda,
  getUserVaultPda,
} from "@/lib/programs/accounts";
import { Address } from "@solana/kit";
import { useWallet } from "@solana/wallet-adapter-react";
import React, { useEffect, useState } from "react";

const Accounts = () => {
  const [userAccounts, setUserAccounts] = useState<Address[]>([]);
  const wallet = useWallet();

  const fetchUserAccounts = async () => {
    if (!wallet.publicKey) return;
    const [vaultPda, lamportsPda] = await Promise.all([
      getUserVaultPda(wallet.publicKey.toString()),
      getUserVaultLamportsPda(wallet.publicKey.toString()),
    ]);

    console.log({ vaultPda, lamportsPda });
    setUserAccounts([vaultPda[0], lamportsPda[0]]);
  };

  useEffect(() => {
    fetchUserAccounts();
  }, [wallet.connected]);
  return (
    <div>
      Accounts :{" "}
      {userAccounts.map((i, k) => (
        <p className="text-sm font-semibold" key={k}>
          {" "}
          {i}{" "}
        </p>
      ))}
    </div>
  );
};

export default Accounts;
