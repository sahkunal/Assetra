import { BN } from "@coral-xyz/anchor";
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";

export const short = (a: PublicKey | string, n = 4) => {
  const s = a.toString();
  return `${s.slice(0, n)}…${s.slice(-n)}`;
};

/** lamports -> "0.0123 SOL" (trims trailing zeros, keeps tiny values visible). */
export function sol(lamports: BN | bigint | number, withUnit = true): string {
  const v = Number(lamports.toString()) / LAMPORTS_PER_SOL;
  let s: string;
  if (v === 0) s = "0";
  else if (v >= 1) s = v.toFixed(3);
  else s = v.toPrecision(3);
  s = s.includes("e") ? v.toFixed(9) : s;
  s = s.includes(".") ? s.replace(/0+$/, "").replace(/\.$/, "") : s;
  return withUnit ? `${s} SOL` : s;
}

export const solToLamports = (value: string): BN => {
  const n = Number(value);
  if (!Number.isFinite(n) || n <= 0) throw new Error("Enter an amount greater than zero");
  return new BN(Math.round(n * LAMPORTS_PER_SOL));
};

export const pct = (bps: number) => `${(bps / 100).toFixed(bps % 100 === 0 ? 0 : 2)}%`;

/** Mirrors buy_tokens: total price in lamports for `amount` tokens. */
export function primaryPrice(
  verifiedRevenue: BN,
  multipleBps: number,
  tokenizedBps: number,
  tokenSupply: BN,
  amount: BN
): bigint {
  const num =
    BigInt(verifiedRevenue.toString()) *
    BigInt(multipleBps) *
    BigInt(tokenizedBps) *
    BigInt(amount.toString());
  const den = 100n * 10_000n * BigInt(tokenSupply.toString());
  return num / den;
}

const PRECISION = 1_000_000_000_000n;

/** Mirrors HolderPosition::settle: rewards claimable right now. */
export function pendingRewards(
  balance: bigint,
  accumulator: BN,
  rewardDebt: BN,
  unclaimed: BN
): bigint {
  const accrued = balance * BigInt(accumulator.toString());
  const debt = BigInt(rewardDebt.toString());
  const fresh = accrued > debt ? (accrued - debt) / PRECISION : 0n;
  return BigInt(unclaimed.toString()) + fresh;
}

/** Friendly message from an Anchor / wallet error. */
export function errorMessage(e: unknown): string {
  const err = e as {
    error?: { errorMessage?: string };
    message?: string;
    logs?: string[];
  };
  if (err?.error?.errorMessage) return err.error.errorMessage;
  const msg = err?.message ?? String(e);
  if (/User rejected/i.test(msg)) return "Transaction was cancelled in the wallet.";
  if (/0x1\b|insufficient lamports/i.test(msg)) return "Not enough SOL in your wallet.";
  return msg.length > 220 ? msg.slice(0, 220) + "…" : msg;
}
