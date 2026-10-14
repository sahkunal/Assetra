"use client";

import Link from "next/link";
import AppShell from "@/components/AppShell";
import Cover from "@/components/Cover";
import { Empty, Panel } from "@/components/ui";
import { loadTracks, tokenBalance } from "@/lib/data";
import { pct, primaryPrice, short, sol } from "@/lib/format";
import { useLoad, useProgram } from "@/lib/hooks";
import { BN } from "@coral-xyz/anchor";
import type { TrackEntry } from "@/lib/program";

interface Row extends TrackEntry {
  remaining: bigint;
}

export default function Marketplace() {
  const { program, connection } = useProgram();

  const { data, loading, error } = useLoad<Row[]>(async () => {
    const tracks = (await loadTracks(program)).filter((t) => t.status === "minted");
    return Promise.all(
      tracks.map(async (t) => ({
        ...t,
        remaining: t.account.vault ? await tokenBalance(connection, t.account.vault) : 0n,
      }))
    );
  }, [program, connection]);

  return (
    <AppShell
      title="Marketplace"
      subtitle="Verified tracks open for investment. Each token is a proportional share of the royalty income."
    >
      {loading && <p className="text-white/60">Loading tracks from devnet…</p>}
      {error && <p className="text-red-300">{error}</p>}
      {data && data.length === 0 && (
        <Empty title="No live tracks yet">A creator has to list a track and have it verified first.</Empty>
      )}

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {data?.map((t) => {
          const a = t.account;
          const supply = BigInt(a.tokenSupply.toString());
          const sold = supply - t.remaining;
          const unitPrice = primaryPrice(
            a.verifiedAnnualRevenue,
            a.valuationMultipleBps,
            a.tokenizedBps,
            a.tokenSupply,
            new BN(1_000_000)
          ) / 1_000_000n;
          const soldPct = supply > 0n ? Number((sold * 100n) / supply) : 0;
          return (
            <Link key={t.address.toString()} href={`/track/${t.address.toString()}`}>
              <Panel className="h-full !p-0 transition hover:border-pop">
                <Cover uri={a.coverArtUri} title={a.title} className="aspect-[4/3] w-full" />
                <div className="p-5 sm:p-6">
                <p className="text-sm text-white/55">{a.isrc}</p>
                <h3 className="mt-2 text-3xl font-medium leading-[1] tracking-[-0.04em]">{a.title}</h3>
                <p className="mt-1 text-sm text-white/55">by {short(a.creator)}</p>

                <dl className="mt-8 grid grid-cols-2 gap-y-4 text-sm">
                  <div>
                    <dt className="text-white/50">Royalties tokenized</dt>
                    <dd className="mt-0.5 text-lg">{pct(a.tokenizedBps)}</dd>
                  </div>
                  <div>
                    <dt className="text-white/50">Price per token</dt>
                    <dd className="mt-0.5 text-lg">{unitPrice.toString()} lamports</dd>
                  </div>
                  <div>
                    <dt className="text-white/50">Verified revenue / yr</dt>
                    <dd className="mt-0.5 text-lg">{sol(a.verifiedAnnualRevenue)}</dd>
                  </div>
                  <div>
                    <dt className="text-white/50">Available</dt>
                    <dd className="mt-0.5 text-lg">{t.remaining.toString()}</dd>
                  </div>
                </dl>

                <div className="mt-6 h-1 w-full bg-white/10">
                  <div className="h-1 bg-pop" style={{ width: `${soldPct}%` }} />
                </div>
                <p className="mt-2 text-xs text-white/50">{soldPct}% sold</p>
                </div>
              </Panel>
            </Link>
          );
        })}
      </div>
    </AppShell>
  );
}
