"use client";

import { BN } from "@coral-xyz/anchor";
import Link from "next/link";
import { useState } from "react";
import AppShell from "@/components/AppShell";
import Cover from "@/components/Cover";
import { Button, Empty, Field, NeedWallet, Panel, TxResult, inputClass } from "@/components/ui";
import { loadListings, loadPositions, PositionView } from "@/lib/data";
import { pendingRewards, short, sol, solToLamports } from "@/lib/format";
import { useAction, useLoad, useProgram } from "@/lib/hooks";
import {
  ata,
  holderPositionPda,
  listingPda,
  mintPda,
  revenuePoolPda,
  TOKEN_2022,
} from "@/lib/program";

export default function Portfolio() {
  const { program, connection, publicKey } = useProgram();

  const { data, loading, error, reload } = useLoad(async () => {
    if (!publicKey) return null;
    const [positions, listings] = await Promise.all([
      loadPositions(program, connection, publicKey),
      loadListings(program, { seller: publicKey }),
    ]);
    return { positions, listings };
  }, [program, connection, publicKey]);

  if (!publicKey)
    return (
      <AppShell title="Portfolio" subtitle="Your royalty tokens and what they've earned.">
        <NeedWallet />
      </AppShell>
    );

  const totalPending = (data?.positions ?? []).reduce(
    (sum, p) => sum + pendingRewards(p.balance, p.accumulator, p.rewardDebt, p.unclaimed),
    0n
  );

  return (
    <AppShell title="Portfolio" subtitle="Your royalty tokens and what they've earned.">
      {loading && !data && <p className="text-white/60">Loading your positions…</p>}
      {error && <p className="text-red-300">{error}</p>}

      {data && (
        <>
          <Panel className="mb-8 flex flex-wrap items-end justify-between gap-4">
            <div>
              <p className="text-sm text-white/55">Royalties ready to claim</p>
              <p className="mt-1 text-4xl font-medium tracking-[-0.04em]">{sol(totalPending)}</p>
            </div>
            <p className="text-sm text-white/50">{data.positions.length} track(s) held</p>
          </Panel>

          {data.positions.length === 0 ? (
            <Empty title="You don't hold any tokens yet">
              <Link href="/marketplace" className="underline underline-offset-4">
                Browse the marketplace
              </Link>
            </Empty>
          ) : (
            <div className="grid gap-4">
              {data.positions.map((p) => (
                <PositionCard
                  key={p.track.address.toString()}
                  p={p}
                  onChanged={reload}
                />
              ))}
            </div>
          )}

          {data.listings.length > 0 && (
            <section className="mt-12">
              <h2 className="text-3xl font-medium tracking-[-0.04em]">Your open listings</h2>
              <div className="mt-5 border border-white/15">
                {data.listings.map((l) => (
                  <ListingRow key={l.address.toString()} l={l} onChanged={reload} />
                ))}
              </div>
            </section>
          )}
        </>
      )}
    </AppShell>
  );
}

function PositionCard({ p, onChanged }: { p: PositionView; onChanged: () => void }) {
  const { program, publicKey } = useProgram();
  const a = p.track.account;
  const pending = pendingRewards(p.balance, p.accumulator, p.rewardDebt, p.unclaimed);
  const mint = mintPda(p.track.address);

  const claim = useAction(onChanged);
  const list = useAction(onChanged);
  const [amount, setAmount] = useState("");
  const [price, setPrice] = useState("");

  const onClaim = () =>
    claim.run(() =>
      program.methods
        .claimRevenue()
        .accounts({
          holder: publicKey!,
          track: p.track.address,
          holderTokenAccount: ata(publicKey!, mint),
          revenuePool: revenuePoolPda(p.track.address),
          holderPosition: holderPositionPda(p.track.address, publicKey!),
        })
        .rpc()
    );

  const onList = () =>
    list.run(async () => {
      const amt = new BN(amount);
      const nonce = new BN(Date.now());
      return program.methods
        .listResale(amt, solToLamports(price), nonce)
        .accounts({
          seller: publicKey!,
          track: p.track.address,
          sellerTokenAccount: ata(publicKey!, mint),
          mint,
          listing: listingPda(p.track.address, publicKey!, nonce),
          tokenProgram: TOKEN_2022,
        })
        .rpc();
    });

  const validList = /^\d+$/.test(amount) && BigInt(amount || "0") > 0n && BigInt(amount || "0") <= p.balance && Number(price) > 0;

  return (
    <Panel>
      <div className="flex flex-wrap items-start justify-between gap-6">
        <Cover uri={a.coverArtUri} title={a.title} className="h-20 w-20 shrink-0 border border-white/15" />
        <div className="min-w-0 flex-1">
          <Link href={`/track/${p.track.address.toString()}`} className="text-3xl font-medium tracking-[-0.04em] hover:underline">
            {a.title}
          </Link>
          <p className="mt-1 text-sm text-white/50">{a.isrc}</p>
        </div>
        <dl className="grid grid-cols-3 gap-8 text-sm">
          <div>
            <dt className="text-white/50">Tokens</dt>
            <dd className="mt-1 text-xl">{p.balance.toString()}</dd>
          </div>
          <div>
            <dt className="text-white/50">Claimable</dt>
            <dd className="mt-1 text-xl">{sol(pending)}</dd>
          </div>
          <div>
            <dt className="text-white/50">Claimed</dt>
            <dd className="mt-1 text-xl">{sol(p.totalClaimed)}</dd>
          </div>
        </dl>
      </div>

      <div className="mt-6 grid gap-6 border-t border-white/10 pt-6 md:grid-cols-2">
        <div>
          <Button disabled={claim.pending || pending === 0n} onClick={onClaim}>
            {claim.pending ? "Confirm in wallet…" : "Claim royalties"}
          </Button>
          <TxResult error={claim.error} sig={claim.sig} />
        </div>

        <div>
          <p className="text-sm text-white/60">Sell on the secondary market</p>
          <div className="mt-2 grid grid-cols-2 gap-3">
            <Field label="Tokens">
              <input className={inputClass} inputMode="numeric" value={amount} placeholder={p.balance.toString()}
                onChange={(e) => setAmount(e.target.value.replace(/\D/g, ""))} />
            </Field>
            <Field label="Price per token (SOL)">
              <input className={inputClass} inputMode="decimal" value={price} placeholder="0.00001"
                onChange={(e) => setPrice(e.target.value.replace(/[^\d.]/g, ""))} />
            </Field>
          </div>
          <Button variant="ghost" className="mt-3" disabled={!validList || list.pending} onClick={onList}>
            {list.pending ? "Confirm in wallet…" : "List for sale"}
          </Button>
          <TxResult error={list.error} sig={list.sig} />
        </div>
      </div>
    </Panel>
  );
}

function ListingRow({
  l,
  onChanged,
}: {
  l: { address: import("@solana/web3.js").PublicKey; account: import("@/lib/program").ListingAccount };
  onChanged: () => void;
}) {
  const { program, publicKey } = useProgram();
  const cancel = useAction(onChanged);
  const mint = mintPda(l.account.track);

  return (
    <div className="flex flex-wrap items-center justify-between gap-4 border-b border-white/10 p-4 last:border-b-0">
      <div>
        <p className="text-lg">
          {l.account.amount.toString()} tokens at {sol(l.account.pricePerToken)} each
        </p>
        <p className="text-sm text-white/50">track {short(l.account.track)}</p>
      </div>
      <div className="text-right">
        <Button
          variant="ghost"
          disabled={cancel.pending}
          onClick={() =>
            cancel.run(() =>
              program.methods
                .cancelResale()
                .accounts({
                  seller: publicKey!,
                  track: l.account.track,
                  sellerTokenAccount: ata(publicKey!, mint),
                  listing: l.address,
                  tokenProgram: TOKEN_2022,
                })
                .rpc()
            )
          }
        >
          {cancel.pending ? "Confirm…" : "Cancel"}
        </Button>
        <TxResult error={cancel.error} sig={cancel.sig} />
      </div>
    </div>
  );
}
