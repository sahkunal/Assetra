"use client";

import { BN } from "@coral-xyz/anchor";
import Link from "next/link";
import { useState } from "react";
import AppShell from "@/components/AppShell";
import Cover from "@/components/Cover";
import {
  Button,
  Empty,
  Field,
  NeedWallet,
  Panel,
  StatusBadge,
  TxResult,
  inputClass,
} from "@/components/ui";
import { loadTracks } from "@/lib/data";
import { pct, sol, solToLamports } from "@/lib/format";
import { useAction, useLoad, useProgram } from "@/lib/hooks";
import {
  creatorProfilePda,
  mintPda,
  revenuePoolPda,
  TOKEN_2022,
  trackPda,
  vaultPda,
} from "@/lib/program";
import type { TrackEntry } from "@/lib/program";

export default function Studio() {
  const { program, publicKey } = useProgram();

  const { data, loading, error, reload } = useLoad(async () => {
    if (!publicKey) return null;
    const profile = await (program.account as any).creatorProfile.fetchNullable(
      creatorProfilePda(publicKey)
    );
    const tracks = profile ? await loadTracks(program, { creator: publicKey }) : [];
    const pools = await Promise.all(
      tracks.map((t) =>
        t.status === "minted"
          ? (program.account as any).revenuePool.fetchNullable(revenuePoolPda(t.address))
          : Promise.resolve(null)
      )
    );
    return { profile, tracks, pools };
  }, [program, publicKey]);

  const subtitle = "List a song, get it verified, mint its tokens and deposit royalties.";

  if (!publicKey)
    return (
      <AppShell title="Creator studio" subtitle={subtitle}>
        <NeedWallet />
      </AppShell>
    );

  return (
    <AppShell title="Creator studio" subtitle={subtitle}>
      {loading && !data && <p className="text-white/60">Loading your studio…</p>}
      {error && <p className="text-red-300">{error}</p>}

      {data && !data.profile && <Register onDone={reload} />}

      {data?.profile && (
        <>
          <p className="mb-8 text-white/60">
            Signed in as <span className="text-white">{data.profile.displayName}</span> ·{" "}
            {data.tracks.length} track(s)
          </p>

          <h2 className="text-3xl font-medium tracking-[-0.04em]">Your tracks</h2>
          <div className="mt-5 grid gap-4">
            {data.tracks.length === 0 && <Empty title="No tracks yet">Create your first one below.</Empty>}
            {data.tracks.map((t, i) => (
              <TrackCard key={t.address.toString()} t={t} pool={data.pools[i]} onChanged={reload} />
            ))}
          </div>

          <NewTrack trackCount={data.profile.trackCount as BN} onDone={reload} />
        </>
      )}
    </AppShell>
  );
}

// ------------------------------------------------------------------ register
function Register({ onDone }: { onDone: () => void }) {
  const { program, publicKey } = useProgram();
  const [name, setName] = useState("");
  const [attest, setAttest] = useState("I hold the rights to the works I list on Assetra.");
  const act = useAction(onDone);

  return (
    <Panel className="max-w-xl">
      <h2 className="text-3xl font-medium tracking-[-0.04em]">Become a creator</h2>
      <p className="mt-2 text-white/60">One-time setup. Your wallet becomes your artist identity.</p>
      <div className="mt-6 grid gap-4">
        <Field label="Artist name" hint="Up to 64 characters">
          <input className={inputClass} maxLength={64} value={name} onChange={(e) => setName(e.target.value)} />
        </Field>
        <Field label="Rights statement" hint="Up to 280 characters. Self-attested.">
          <textarea
            className={`${inputClass} min-h-24`}
            maxLength={280}
            value={attest}
            onChange={(e) => setAttest(e.target.value)}
          />
        </Field>
      </div>
      <Button
        className="mt-6"
        disabled={act.pending || !name.trim()}
        onClick={() =>
          act.run(() =>
            program.methods
              .registerCreator(name.trim(), attest.trim())
              .accounts({ authority: publicKey!, creatorProfile: creatorProfilePda(publicKey!) })
              .rpc()
          )
        }
      >
        {act.pending ? "Confirm in wallet…" : "Register"}
      </Button>
      <TxResult error={act.error} sig={act.sig} />
    </Panel>
  );
}

// ----------------------------------------------------------------- new track
function NewTrack({ trackCount, onDone }: { trackCount: BN; onDone: () => void }) {
  const { program, publicKey } = useProgram();
  const [f, setF] = useState({
    title: "",
    isrc: "",
    cover: "",
    preview: "",
    revenue: "0.1",
    percent: "20",
    multiple: "5",
    supply: "100000",
  });
  const set = (k: keyof typeof f) => (e: { target: { value: string } }) =>
    setF((s) => ({ ...s, [k]: e.target.value }));
  const act = useAction(onDone);

  const valid =
    f.title.trim() &&
    f.isrc.trim() &&
    Number(f.revenue) > 0 &&
    Number(f.percent) > 0 &&
    Number(f.percent) <= 100 &&
    Number(f.multiple) > 0 &&
    /^\d+$/.test(f.supply) &&
    Number(f.supply) > 0;

  const submit = () =>
    act.run(() =>
      program.methods
        .createTrack(
          f.title.trim(),
          f.isrc.trim(),
          f.cover.trim(),
          f.preview.trim(),
          solToLamports(f.revenue),
          Math.round(Number(f.percent) * 100),
          Math.round(Number(f.multiple) * 100),
          new BN(f.supply)
        )
        .accounts({
          authority: publicKey!,
          creatorProfile: creatorProfilePda(publicKey!),
          track: trackPda(publicKey!, trackCount),
        })
        .rpc()
    );

  return (
    <section className="mt-14">
      <h2 className="text-3xl font-medium tracking-[-0.04em]">List a new track</h2>
      <Panel className="mt-5">
        <div className="grid gap-4 md:grid-cols-2">
          <Field label="Title">
            <input className={inputClass} maxLength={100} value={f.title} onChange={set("title")} />
          </Field>
          <Field label="ISRC" hint="Up to 15 characters">
            <input className={inputClass} maxLength={15} value={f.isrc} onChange={set("isrc")} placeholder="USRC17600001" />
          </Field>
          <Field
            label="Cover art URL"
            hint="A direct link to an image file (.jpg, .png, .webp) or an ipfs:// link. Shown in the marketplace."
          >
            <div className="flex items-start gap-3">
              <input
                className={inputClass}
                value={f.cover}
                onChange={set("cover")}
                placeholder="https://…/cover.jpg"
              />
              <Cover uri={f.cover} title={f.title} className="h-[46px] w-[46px] shrink-0 border border-white/15" />
            </div>
          </Field>
          <Field
            label="Audio preview URL (optional)"
            hint="A YouTube link or a direct .mp3 file link."
          >
            <input
              className={inputClass}
              value={f.preview}
              onChange={set("preview")}
              placeholder="https://…/preview.mp3"
            />
          </Field>
          <Field label="Annual royalty revenue (SOL)" hint="What the song earns per year. A verifier confirms it.">
            <input className={inputClass} inputMode="decimal" value={f.revenue} onChange={set("revenue")} />
          </Field>
          <Field label="Share of royalties to tokenize (%)">
            <input className={inputClass} inputMode="decimal" value={f.percent} onChange={set("percent")} />
          </Field>
          <Field label="Valuation multiple (×)" hint="Price = annual revenue × multiple × share">
            <input className={inputClass} inputMode="decimal" value={f.multiple} onChange={set("multiple")} />
          </Field>
          <Field label="Token supply">
            <input
              className={inputClass}
              inputMode="numeric"
              value={f.supply}
              onChange={(e) => setF((s) => ({ ...s, supply: e.target.value.replace(/\D/g, "") }))}
            />
          </Field>
        </div>
        <Button className="mt-6" disabled={!valid || act.pending} onClick={submit}>
          {act.pending ? "Confirm in wallet…" : "Create track"}
        </Button>
        <TxResult error={act.error} sig={act.sig} />
      </Panel>
    </section>
  );
}

// --------------------------------------------------------------- track card
function TrackCard({
  t,
  pool,
  onChanged,
}: {
  t: TrackEntry;
  pool: { totalDeposited: BN } | null;
  onChanged: () => void;
}) {
  const { program, publicKey } = useProgram();
  const a = t.account;
  const [deposit, setDeposit] = useState("0.01");
  const mintAct = useAction(onChanged);
  const poolAct = useAction(onChanged);
  const depAct = useAction(onChanged);

  return (
    <Panel>
      <div className="flex flex-wrap items-start justify-between gap-4">
        <Cover uri={a.coverArtUri} title={a.title} className="h-20 w-20 shrink-0 border border-white/15" />
        <div className="min-w-0 flex-1">
          <Link href={`/track/${t.address.toString()}`} className="text-2xl font-medium tracking-[-0.03em] hover:underline">
            {a.title}
          </Link>
          <p className="mt-1 text-sm text-white/50">
            {a.isrc} · {pct(a.tokenizedBps)} tokenized · supply {a.tokenSupply.toString()} · declared{" "}
            {sol(a.declaredAnnualRevenue)}/yr
          </p>
        </div>
        <StatusBadge status={t.status} />
      </div>

      <div className="mt-5 border-t border-white/10 pt-5">
        {t.status === "configured" && (
          <p className="text-white/60">
            Waiting for the verifier to approve this track. You&apos;ll be able to mint its tokens next.
          </p>
        )}

        {t.status === "verified" && (
          <>
            <p className="mb-3 text-white/60">
              Verified at {sol(a.verifiedAnnualRevenue)}/yr. Mint the {a.tokenSupply.toString()} tokens to open
              it for sale.
            </p>
            <Button
              disabled={mintAct.pending}
              onClick={() =>
                mintAct.run(() =>
                  program.methods
                    .mintTrackTokens()
                    .accounts({
                      authority: publicKey!,
                      track: t.address,
                      mint: mintPda(t.address),
                      vault: vaultPda(t.address),
                      tokenProgram: TOKEN_2022,
                    })
                    .rpc()
                )
              }
            >
              {mintAct.pending ? "Confirm in wallet…" : "Mint tokens"}
            </Button>
            <TxResult error={mintAct.error} sig={mintAct.sig} />
          </>
        )}

        {t.status === "minted" && !pool && (
          <>
            <p className="mb-3 text-white/60">
              Tokens are minted. Open the revenue pool so royalties can be deposited and sold shares can claim.
            </p>
            <Button
              disabled={poolAct.pending}
              onClick={() =>
                poolAct.run(() =>
                  program.methods
                    .initializeRevenuePool()
                    .accounts({
                      authority: publicKey!,
                      track: t.address,
                      revenuePool: revenuePoolPda(t.address),
                    })
                    .rpc()
                )
              }
            >
              {poolAct.pending ? "Confirm in wallet…" : "Open revenue pool"}
            </Button>
            <TxResult error={poolAct.error} sig={poolAct.sig} />
          </>
        )}

        {t.status === "minted" && pool && (
          <>
            <p className="mb-4 text-white/60">
              Total deposited so far: <span className="text-white">{sol(pool.totalDeposited)}</span>
            </p>
            <div className="flex flex-wrap items-end gap-4">
              <div className="w-48">
                <Field label="Deposit royalties (SOL)">
                  <input
                    className={inputClass}
                    inputMode="decimal"
                    value={deposit}
                    onChange={(e) => setDeposit(e.target.value.replace(/[^\d.]/g, ""))}
                  />
                </Field>
              </div>
              <Button
                disabled={depAct.pending || !(Number(deposit) > 0)}
                onClick={() =>
                  depAct.run(() =>
                    program.methods
                      .depositRevenue(solToLamports(deposit))
                      .accounts({
                        authority: publicKey!,
                        track: t.address,
                        revenuePool: revenuePoolPda(t.address),
                      })
                      .rpc()
                  )
                }
              >
                {depAct.pending ? "Confirm in wallet…" : "Deposit"}
              </Button>
            </div>
            <TxResult error={depAct.error} sig={depAct.sig} />
          </>
        )}
      </div>
    </Panel>
  );
}
