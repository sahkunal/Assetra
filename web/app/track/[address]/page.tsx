"use client";

import { BN } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { useParams } from "next/navigation";
import { useMemo, useState } from "react";
import AppShell from "@/components/AppShell";
import Cover, { resolveUri, youtubeId } from "@/components/Cover";
import { BackLink, Button, Empty, Field, Panel, StatusBadge, TxResult, inputClass } from "@/components/ui";
import { holderBalance, loadListings, loadTrack, tokenBalance } from "@/lib/data";
import { pct, primaryPrice, short, sol } from "@/lib/format";
import { useAction, useLoad, useProgram } from "@/lib/hooks";
import {
  ata,
  explorerAddress,
  holderPositionPda,
  listingPda,
  mintPda,
  revenuePoolPda,
  TOKEN_2022,
  vaultPda,
} from "@/lib/program";

export default function TrackPage() {
  const params = useParams<{ address: string }>();
  const { program, connection, publicKey } = useProgram();

  const address = useMemo(() => {
    try {
      return new PublicKey(params.address);
    } catch {
      return null;
    }
  }, [params.address]);

  const { data, loading, error, reload } = useLoad(async () => {
    if (!address) return null;
    const track = await loadTrack(program, address);
    if (!track) return null;
    const [remaining, mine, listings] = await Promise.all([
      track.account.vault ? tokenBalance(connection, track.account.vault) : Promise.resolve(0n),
      publicKey && track.account.mint
        ? holderBalance(connection, track.account.mint, publicKey)
        : Promise.resolve(0n),
      track.status === "minted" ? loadListings(program, { track: address }) : Promise.resolve([]),
    ]);
    return { track, remaining, mine, listings };
  }, [program, connection, address, publicKey]);

  const [amount, setAmount] = useState("100");
  const buy = useAction(reload);
  const resale = useAction(reload);

  if (!address) return <AppShell title="Track"><Empty title="Invalid track address" /></AppShell>;
  if (loading && !data)
    return <AppShell title="Track"><p className="text-white/60">Loading…</p></AppShell>;
  if (error || !data)
    return <AppShell title="Track"><Empty title="Track not found">{error}</Empty></AppShell>;

  const { track, remaining, mine, listings } = data;
  const a = track.account;
  const amt = /^\d+$/.test(amount) ? new BN(amount) : new BN(0);
  const total = primaryPrice(a.verifiedAnnualRevenue, a.valuationMultipleBps, a.tokenizedBps, a.tokenSupply, amt);
  const supply = BigInt(a.tokenSupply.toString());
  const mint = mintPda(track.address);

  const onBuy = () =>
    buy.run(() =>
      program.methods
        .buyTokens(amt)
        .accounts({
          investor: publicKey!,
          creator: a.creator,
          track: track.address,
          vault: vaultPda(track.address),
          investorTokenAccount: ata(publicKey!, mint),
          mint,
          revenuePool: revenuePoolPda(track.address),
          holderPosition: holderPositionPda(track.address, publicKey!),
          tokenProgram: TOKEN_2022,
        })
        .rpc()
    );

  const onBuyResale = (l: (typeof listings)[number]) =>
    resale.run(() =>
      program.methods
        .buyResale()
        .accounts({
          buyer: publicKey!,
          seller: l.account.seller,
          track: track.address,
          sellerTokenAccount: ata(l.account.seller, mint),
          buyerTokenAccount: ata(publicKey!, mint),
          mint,
          revenuePool: revenuePoolPda(track.address),
          sellerHolderPosition: holderPositionPda(track.address, l.account.seller),
          buyerHolderPosition: holderPositionPda(track.address, publicKey!),
          listing: listingPda(track.address, l.account.seller, l.account.nonce),
          tokenProgram: TOKEN_2022,
        })
        .rpc()
    );

  const onCancel = (l: (typeof listings)[number]) =>
    resale.run(() =>
      program.methods
        .cancelResale()
        .accounts({
          seller: publicKey!,
          track: track.address,
          sellerTokenAccount: ata(publicKey!, mint),
          listing: listingPda(track.address, publicKey!, l.account.nonce),
          tokenProgram: TOKEN_2022,
        })
        .rpc()
    );

  return (
    <AppShell title={a.title} subtitle={`ISRC ${a.isrc} · creator ${short(a.creator)}`}>
      <div className="mb-8 flex items-center gap-4">
        <BackLink href="/marketplace">Marketplace</BackLink>
        <StatusBadge status={track.status} />
      </div>

      <div className="mb-4 grid gap-4 md:grid-cols-[minmax(0,20rem)_1fr]">
        <Cover uri={a.coverArtUri} title={a.title} className="aspect-square w-full border border-white/15" />
        <Panel className="flex flex-col justify-end">
          <p className="text-sm text-white/55">Audio preview</p>
          {youtubeId(a.audioPreviewUri) ? (
            <iframe
              title={`${a.title} preview`}
              src={`https://www.youtube-nocookie.com/embed/${youtubeId(a.audioPreviewUri)}`}
              allow="encrypted-media; picture-in-picture"
              allowFullScreen
              className="mt-3 aspect-video w-full border border-white/15"
            />
          ) : resolveUri(a.audioPreviewUri) ? (
            // eslint-disable-next-line jsx-a11y/media-has-caption
            <audio controls preload="none" src={resolveUri(a.audioPreviewUri)!} className="mt-3 w-full" />
          ) : (
            <p className="mt-3 text-white/50">No preview provided.</p>
          )}
          <p className="mt-6 text-sm text-white/50">
            Investors buy a proportional claim on this song&apos;s future royalty income, paid out on-chain.
          </p>
        </Panel>
      </div>

      <div className="grid gap-4 lg:grid-cols-3">
        <Panel className="lg:col-span-2">
          <dl className="grid grid-cols-2 gap-y-6 text-sm sm:grid-cols-3">
            <Stat label="Royalties tokenized" value={pct(a.tokenizedBps)} />
            <Stat label="Verified revenue / yr" value={track.status === "configured" ? "—" : sol(a.verifiedAnnualRevenue)} />
            <Stat label="Declared revenue / yr" value={sol(a.declaredAnnualRevenue)} />
            <Stat label="Token supply" value={supply.toString()} />
            <Stat label="Available" value={remaining.toString()} />
            <Stat label="You hold" value={mine.toString()} />
          </dl>
          <p className="mt-8 text-sm text-white/50">
            Each token is {(a.tokenizedBps / 100 / Number(supply)).toFixed(5)}% of the song&apos;s
            royalty income. Holders are paid pro-rata whenever royalties are deposited.
          </p>
          <a
            className="mt-4 inline-block text-sm underline underline-offset-4"
            href={explorerAddress(track.address)}
            target="_blank"
            rel="noreferrer"
          >
            Track account on Explorer ↗
          </a>
        </Panel>

        <Panel>
          <h2 className="text-2xl font-medium tracking-[-0.03em]">Buy tokens</h2>
          {track.status !== "minted" ? (
            <p className="mt-3 text-white/60">Not open for sale yet.</p>
          ) : (
            <>
              <div className="mt-5">
                <Field label="Amount" hint={`Up to ${remaining.toString()} available`}>
                  <input
                    className={inputClass}
                    inputMode="numeric"
                    value={amount}
                    onChange={(e) => setAmount(e.target.value.replace(/\D/g, ""))}
                  />
                </Field>
              </div>
              <p className="mt-4 text-sm text-white/60">
                Total <span className="text-white">{sol(total)}</span>
              </p>
              <Button
                className="mt-5 w-full"
                disabled={!publicKey || buy.pending || amt.isZero() || amt.gt(new BN(remaining.toString()))}
                onClick={onBuy}
              >
                {!publicKey ? "Connect wallet" : buy.pending ? "Confirm in wallet…" : "Buy"}
              </Button>
              <TxResult error={buy.error} sig={buy.sig} />
            </>
          )}
        </Panel>
      </div>

      {track.status === "minted" && (
        <section className="mt-12">
          <h2 className="text-3xl font-medium tracking-[-0.04em]">Resale listings</h2>
          {listings.length === 0 ? (
            <p className="mt-4 text-white/60">No holders are selling right now.</p>
          ) : (
            <div className="mt-5 border border-white/15">
              {listings.map((l) => {
                const mineListing = publicKey?.equals(l.account.seller);
                const totalPrice = l.account.pricePerToken.mul(l.account.amount);
                return (
                  <div
                    key={l.address.toString()}
                    className="flex flex-wrap items-center justify-between gap-4 border-b border-white/10 p-4 last:border-b-0"
                  >
                    <div>
                      <p className="text-lg">
                        {l.account.amount.toString()} tokens at {sol(l.account.pricePerToken)} each
                      </p>
                      <p className="text-sm text-white/50">
                        seller {short(l.account.seller)} · total {sol(totalPrice)}
                      </p>
                    </div>
                    {mineListing ? (
                      <Button variant="ghost" disabled={resale.pending} onClick={() => onCancel(l)}>
                        Cancel
                      </Button>
                    ) : (
                      <Button disabled={!publicKey || resale.pending} onClick={() => onBuyResale(l)}>
                        {resale.pending ? "Confirm…" : "Buy listing"}
                      </Button>
                    )}
                  </div>
                );
              })}
            </div>
          )}
          <TxResult error={resale.error} sig={resale.sig} />
        </section>
      )}
    </AppShell>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-white/50">{label}</dt>
      <dd className="mt-1 text-xl tracking-[-0.02em]">{value}</dd>
    </div>
  );
}
