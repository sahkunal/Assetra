"use client";

import { useWallet } from "@solana/wallet-adapter-react";
import { useEffect, useRef, useState } from "react";
import { short } from "@/lib/format";

export default function ConnectButton({ className = "" }: { className?: string }) {
  const { wallets, wallet, publicKey, connecting, connected, select, connect, disconnect } =
    useWallet();
  const [open, setOpen] = useState(false);
  const [pendingConnect, setPendingConnect] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  // `select` is async in the adapter; connect once the choice has applied.
  useEffect(() => {
    if (pendingConnect && wallet && !connected && !connecting) {
      setPendingConnect(false);
      connect().catch(() => undefined);
    }
  }, [pendingConnect, wallet, connected, connecting, connect]);

  useEffect(() => {
    const onDown = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", onDown);
    return () => document.removeEventListener("mousedown", onDown);
  }, []);

  const base =
    "pixel-cut px-5 py-2.5 text-base font-medium tracking-[-0.01em] transition hover:brightness-110";

  if (connected && publicKey) {
    return (
      <div ref={ref} className={`relative ${className}`}>
        <button onClick={() => setOpen((v) => !v)} className={`${base} border border-white/50 bg-ink text-white`}>
          {short(publicKey)}
        </button>
        {open && (
          <div className="absolute right-0 z-30 mt-2 w-48 border border-white/20 bg-ink p-1 text-sm">
            <button
              onClick={() => {
                setOpen(false);
                disconnect();
              }}
              className="w-full px-3 py-2 text-left hover:bg-white/10"
            >
              Disconnect
            </button>
          </div>
        )}
      </div>
    );
  }

  const installed = wallets.filter((w) => w.readyState === "Installed");

  return (
    <div ref={ref} className={`relative ${className}`}>
      <button onClick={() => setOpen((v) => !v)} disabled={connecting} className={`${base} bg-pop text-ink`}>
        {connecting ? "Connecting…" : "Connect wallet"}
      </button>
      {open && (
        <div className="absolute right-0 z-30 mt-2 w-64 border border-white/20 bg-ink p-1 text-sm">
          {installed.length === 0 ? (
            <p className="p-3 text-white/70">
              No Solana wallet detected. Install{" "}
              <a className="underline" href="https://phantom.app" target="_blank" rel="noreferrer">
                Phantom
              </a>{" "}
              and set it to Devnet.
            </p>
          ) : (
            installed.map((w) => (
              <button
                key={w.adapter.name}
                onClick={() => {
                  setOpen(false);
                  select(w.adapter.name);
                  setPendingConnect(true);
                }}
                className="flex w-full items-center gap-3 px-3 py-2 text-left hover:bg-white/10"
              >
                {/* eslint-disable-next-line @next/next/no-img-element */}
                <img src={w.adapter.icon} alt="" className="h-5 w-5" />
                {w.adapter.name}
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
}
