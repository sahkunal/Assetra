"use client";

import { ConnectionProvider, WalletProvider } from "@solana/wallet-adapter-react";
import { ReactNode, useMemo } from "react";
import { RPC_URL } from "@/lib/program";

/** Wallet-Standard wallets (Phantom, Solflare, Backpack…) are auto-detected. */
export default function Providers({ children }: { children: ReactNode }) {
  const wallets = useMemo(() => [], []);
  return (
    <ConnectionProvider endpoint={RPC_URL} config={{ commitment: "confirmed" }}>
      <WalletProvider wallets={wallets} autoConnect>
        {children}
      </WalletProvider>
    </ConnectionProvider>
  );
}
