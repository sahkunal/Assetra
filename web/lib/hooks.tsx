"use client";
import type { Wallet } from "@coral-xyz/anchor";

import { useAnchorWallet, useConnection } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { DependencyList, useCallback, useEffect, useMemo, useState } from "react";
import { errorMessage } from "./format";
import { AssetraProgram, configPda, getProgram } from "./program";

export function useProgram() {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  const program = useMemo(
    () => getProgram(connection, (wallet as unknown as Wallet) ?? null),
    [connection, wallet]
  );
  return { program, connection, wallet, publicKey: wallet?.publicKey ?? null };
}

/** Run an async loader, re-run on dependency change or `reload()`. */
export function useLoad<T>(fn: () => Promise<T>, deps: DependencyList) {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [tick, setTick] = useState(0);

  useEffect(() => {
    let alive = true;
    setLoading(true);
    fn()
      .then((d) => alive && (setData(d), setError(null)))
      .catch((e) => alive && setError(errorMessage(e)))
      .finally(() => alive && setLoading(false));
    return () => {
      alive = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [...deps, tick]);

  const reload = useCallback(() => setTick((t) => t + 1), []);
  return { data, error, loading, reload };
}

/** Wraps a transaction: pending / error / signature state for the UI. */
export function useAction(onDone?: () => void) {
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sig, setSig] = useState<string | null>(null);

  const run = useCallback(
    async (fn: () => Promise<string>) => {
      setPending(true);
      setError(null);
      setSig(null);
      try {
        const s = await fn();
        setSig(s);
        onDone?.();
      } catch (e) {
        setError(errorMessage(e));
      } finally {
        setPending(false);
      }
    },
    [onDone]
  );
  return { run, pending, error, sig };
}

// ---- who is the verifier? cached so the menu does not refetch on every page.
let configCache: { verifier: PublicKey; admin: PublicKey } | null = null;

export function useConfig() {
  const { program } = useProgram();
  const [config, setConfig] = useState(configCache);
  useEffect(() => {
    if (configCache) return;
    (program.account as any).programConfig
      .fetchNullable(configPda())
      .then((c: any) => {
        if (c) {
          configCache = { verifier: c.verifier, admin: c.admin };
          setConfig(configCache);
        }
      })
      .catch(() => undefined);
  }, [program]);
  return config;
}

export function useIsVerifier() {
  const { publicKey } = useProgram();
  const config = useConfig();
  return !!(publicKey && config && config.verifier.equals(publicKey));
}

export type { AssetraProgram };
