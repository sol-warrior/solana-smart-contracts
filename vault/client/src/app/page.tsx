import Accounts from "@/components/vault/Accounts";
import { ConnectWallet } from "@/components/wallet/ConnectWallet";
import WalletBalance from "@/components/wallet/WalletBalance";
import Image from "next/image";

export default function Home() {
  return (
    <div className="flex min-h-screen justify-center py-2.5 bg-zinc-50 font-sans dark:bg-black">
      <main className="">
        <h1 className="text-4xl font-semibold text-center py-4 mb-5">
          Anchor Vault{" "}
        </h1>

        <div className="flex items-center justify-center gap-5">
          <WalletBalance />
          <ConnectWallet />
        </div>

        <div className="my-12">
          <Accounts />
        </div>
      </main>
    </div>
  );
}
