"use client";

import Link from "next/link";
import { ButtonHTMLAttributes, ReactNode } from "react";
import { explorerTx } from "@/lib/program";
import type { TrackStatus } from "@/lib/program";

export function Button({
  variant = "primary",
  className = "",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: "primary" | "ghost" }) {
  const styles =
    variant === "primary"
      ? "pixel-cut bg-pop text-ink hover:brightness-110"
      : "rounded-full border border-white/40 hover:bg-white hover:text-ink";
  return (
    <button
      {...props}
      className={`px-5 py-2.5 text-base font-medium tracking-[-0.01em] transition disabled:cursor-not-allowed disabled:opacity-40 ${styles} ${className}`}
    />
  );
}

export function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <label className="block">
      <span className="text-sm text-white/70">{label}</span>
      <div className="mt-1.5">{children}</div>
      {hint && <span className="mt-1 block text-xs text-white/45">{hint}</span>}
    </label>
  );
}

export const inputClass =
  "w-full border border-white/20 bg-white/[0.05] px-3 py-2.5 text-base outline-none transition placeholder:text-white/30 focus:border-pop";

export function Panel({
  children,
  className = "",
}: {
  children: ReactNode;
  className?: string;
}) {
  return <div className={`border border-white/15 p-5 sm:p-6 ${className}`}>{children}</div>;
}

const STATUS_STYLE: Record<TrackStatus, string> = {
  configured: "border-white/30 text-white/70",
  verified: "border-white text-white",
  minted: "border-pop text-pop",
};
const STATUS_LABEL: Record<TrackStatus, string> = {
  configured: "Awaiting verification",
  verified: "Verified",
  minted: "Live",
};

export function StatusBadge({ status }: { status: TrackStatus }) {
  return (
    <span className={`inline-block rounded-full border px-3 py-0.5 text-xs ${STATUS_STYLE[status]}`}>
      {STATUS_LABEL[status]}
    </span>
  );
}

/** Result line shown under an action button. */
export function TxResult({ error, sig }: { error: string | null; sig: string | null }) {
  if (error) return <p className="mt-3 text-sm text-red-300">{error}</p>;
  if (sig)
    return (
      <p className="mt-3 text-sm text-white/70">
        Done.{" "}
        <a className="underline underline-offset-4" href={explorerTx(sig)} target="_blank" rel="noreferrer">
          View transaction ↗
        </a>
      </p>
    );
  return null;
}

export function Empty({ title, children }: { title: string; children?: ReactNode }) {
  return (
    <Panel className="py-14 text-center">
      <p className="text-2xl tracking-[-0.03em]">{title}</p>
      {children && <div className="mt-3 text-white/60">{children}</div>}
    </Panel>
  );
}

export function NeedWallet() {
  return (
    <Empty title="Connect your wallet">
      Use the button in the top right. Set Phantom to <b>Devnet</b>.
    </Empty>
  );
}

export function BackLink({ href, children }: { href: string; children: ReactNode }) {
  return (
    <Link href={href} className="text-sm text-white/60 underline-offset-4 hover:text-white hover:underline">
      ← {children}
    </Link>
  );
}
