"use client";

import { useState } from "react";
import AppShell from "@/components/AppShell";
import Cover, { resolveUri } from "@/components/Cover";
import MediaPreview from "@/components/MediaPreview";
import { Button, Empty, Field, NeedWallet, Panel, TxResult, inputClass } from "@/components/ui";
import { loadTracks } from "@/lib/data";
import { pct, short, sol, solToLamports } from "@/lib/format";
import { useAction, useConfig, useLoad, useProgram } from "@/lib/hooks";
import { configPda, type TrackEntry } from "@/lib/program";

export default function VerifierPage() {
  const { program, publicKey } = useProgram();
  const config = useConfig();

  const { data, loading, error, reload } = useLoad(async () => {
    const all = await loadTracks(program);
    return all.filter((t) => t.status === "configured");
  }, [program]);

  const subtitle = "Approve a track's royalty data so it can be minted and sold.";

  if (!publicKey)
    return (
      <AppShell title="Verifier" subtitle={subtitle}>
        <NeedWallet />
      </AppShell>
    );

  if (config && !config.verifier.equals(publicKey))
    return (
      <AppShell title="Verifier" subtitle={subtitle}>
        <Empty title="This wallet is not the verifier">
          Connect <span className="break-all">{config.verifier.toString()}</span> to approve tracks.
        </Empty>
      </AppShell>
    );

  return (
    <AppShell title="Verifier" subtitle={subtitle}>
      {loading && !data && <p className="text-white/60">Loading the queue…</p>}
      {error && <p className="text-red-300">{error}</p>}
      {data && data.length === 0 && <Empty title="Nothing waiting for review" />}
      <div className="grid gap-4">
        {data?.map((t) => (
          <Review key={t.address.toString()} t={t} onChanged={reload} />
        ))}
      </div>
    </AppShell>
  );
}

function Review({ t, onChanged }: { t: TrackEntry; onChanged: () => void }) {
  const { program, publicKey } = useProgram();
  const a = t.account;
  const [revenue, setRevenue] = useState(
    (Number(a.declaredAnnualRevenue.toString()) / 1_000_000_000).toString()
  );
  const approve = useAction(onChanged);

  return (
    <Panel>
      <div className="flex flex-wrap items-start justify-between gap-6">
        <div>
          <h3 className="text-3xl font-medium tracking-[-0.04em]">{a.title}</h3>
          <p className="mt-1 text-sm text-white/50">
            ISRC {a.isrc} · creator {short(a.creator)}
          </p>
        </div>
        <dl className="grid grid-cols-3 gap-8 text-sm">
          <div>
            <dt className="text-white/50">Declared / yr</dt>
            <dd className="mt-1 text-xl">{sol(a.declaredAnnualRevenue)}</dd>
          </div>
          <div>
            <dt className="text-white/50">Tokenized</dt>
            <dd className="mt-1 text-xl">{pct(a.tokenizedBps)}</dd>
          </div>
          <div>
            <dt className="text-white/50">Supply</dt>
            <dd className="mt-1 text-xl">{a.tokenSupply.toString()}</dd>
          </div>
        </dl>
      </div>

      <div className="mt-6 grid gap-6 border-t border-white/10 pt-6 md:grid-cols-[12rem_1fr]">
        <div>
          <Cover uri={a.coverArtUri} title={a.title} className="aspect-square w-full border border-white/15" />
          <p className="mt-2 text-xs text-white/50">Cover art</p>
          <LinkLine label="Cover link" uri={a.coverArtUri} />
        </div>
        <div>
          <MediaPreview uri={a.audioPreviewUri} title={a.title} />
          <p className="mt-2 text-xs text-white/50">Audio preview</p>
          <LinkLine label="Preview link" uri={a.audioPreviewUri} />
        </div>
      </div>

      <div className="mt-6 flex flex-wrap items-end gap-4 border-t border-white/10 pt-6">
        <div className="w-full max-w-xs">
          <Field label="Verified annual revenue (SOL)" hint="Pricing uses this figure, not the declared one.">
            <input
              className={inputClass}
              inputMode="decimal"
              value={revenue}
              onChange={(e) => setRevenue(e.target.value.replace(/[^\d.]/g, ""))}
            />
          </Field>
        </div>
        <Button
          disabled={approve.pending || !(Number(revenue) > 0)}
          onClick={() =>
            approve.run(() =>
              program.methods
                .verifyTrack(solToLamports(revenue))
                .accounts({
                  verifier: publicKey!,
                  config: configPda(),
                  track: t.address,
                })
                .rpc()
            )
          }
        >
          {approve.pending ? "Confirm in wallet…" : "Approve"}
        </Button>
      </div>
      <TxResult error={approve.error} sig={approve.sig} />
    </Panel>
  );
}

/** The raw URL the creator supplied, so the verifier can inspect the source. */
function LinkLine({ label, uri }: { label: string; uri: string }) {
  const href = resolveUri(uri) ?? (/^https?:\/\//i.test(uri) ? uri : null);
  if (!uri) return <p className="mt-1 text-xs text-white/40">{label}: none</p>;
  return (
    <p className="mt-1 break-all text-xs text-white/50">
      {label}:{" "}
      {href ? (
        <a className="underline underline-offset-2 hover:text-white" href={href} target="_blank" rel="noreferrer">
          {uri}
        </a>
      ) : (
        uri
      )}
    </p>
  );
}
