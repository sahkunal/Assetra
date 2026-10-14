"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import Cover from "@/components/Cover";
import { loadTracks } from "@/lib/data";
import { pct, short } from "@/lib/format";
import { useProgram } from "@/lib/hooks";

interface CardData {
  key: string;
  title: string;
  sub: string;
  cover: string | null;
  href: string | null;
}

// Neutral placeholders, used only if devnet has no live tracks to show.
const SAMPLES: CardData[] = [
  { key: "s1", title: "Midnight Drive", sub: "20% tokenized", cover: null, href: null },
  { key: "s2", title: "Golden Hour", sub: "15% tokenized", cover: null, href: null },
  { key: "s3", title: "Neon Streets", sub: "25% tokenized", cover: null, href: null },
  { key: "s4", title: "Slow Waves", sub: "10% tokenized", cover: null, href: null },
  { key: "s5", title: "Afterglow", sub: "30% tokenized", cover: null, href: null },
];

/** A fan of cards with the active one centered. Shows live tracks from devnet. */
export default function ShowcaseCards() {
  const { program } = useProgram();
  const [cards, setCards] = useState<CardData[]>(SAMPLES);
  const [active, setActive] = useState(0);
  const [paused, setPaused] = useState(false);

  useEffect(() => {
    let alive = true;
    loadTracks(program)
      .then((tracks) => {
        const live = tracks
          .filter((t) => t.status === "minted")
          .slice(0, 5)
          .map<CardData>((t) => ({
            key: t.address.toString(),
            title: t.account.title,
            sub: `${pct(t.account.tokenizedBps)} tokenized · ${short(t.account.creator)}`,
            cover: t.account.coverArtUri || null,
            href: `/track/${t.address.toString()}`,
          }));
        if (alive && live.length > 0) {
          setCards(live);
          setActive(0);
        }
      })
      .catch(() => undefined); // keep the placeholders
    return () => {
      alive = false;
    };
  }, [program]);

  useEffect(() => {
    if (paused || cards.length < 2) return;
    const id = setInterval(() => setActive((a) => (a + 1) % cards.length), 3500);
    return () => clearInterval(id);
  }, [paused, cards.length]);

  const n = cards.length;
  const half = Math.floor(n / 2);

  return (
    <div
      className="relative h-[360px] w-full select-none"
      onMouseEnter={() => setPaused(true)}
      onMouseLeave={() => setPaused(false)}
    >
      {cards.map((c, i) => {
        // signed circular distance from the active card
        const d = ((i - active + n + half) % n) - half;
        const abs = Math.abs(d);
        const centre = d === 0;

        const inner = (
          <div
            className={`flex h-full w-full flex-col overflow-hidden rounded-2xl border bg-white/[0.07] backdrop-blur-sm transition-colors ${
              centre ? "border-white/40" : "border-white/15"
            }`}
          >
            <Cover uri={c.cover} title={c.title} className="aspect-square w-full" />
            <div className="p-3 text-center">
              <p className="truncate text-lg font-medium leading-tight tracking-[-0.02em]">{c.title}</p>
              <p className="mt-0.5 truncate text-xs text-white/60">{c.sub}</p>
            </div>
          </div>
        );

        return (
          <div
            key={c.key}
            onClick={() => !centre && setActive(i)}
            style={{
              transform: `translateX(${d * 54}%) scale(${1 - abs * 0.14})`,
              opacity: abs > 2 ? 0 : 1 - abs * 0.3,
              zIndex: 10 - abs,
              pointerEvents: abs > 2 ? "none" : "auto",
            }}
            className={`absolute left-1/2 top-1/2 -ml-[110px] -mt-[160px] h-[320px] w-[220px] transition-all duration-500 ease-out ${
              centre ? "" : "cursor-pointer"
            }`}
          >
            {centre && c.href ? (
              <Link href={c.href} className="block h-full w-full">
                {inner}
              </Link>
            ) : (
              inner
            )}
          </div>
        );
      })}
    </div>
  );
}
