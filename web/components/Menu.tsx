"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
import ConnectButton from "@/components/ConnectButton";
import { useIsVerifier } from "@/lib/hooks";

const EXPLORER =
  "https://explorer.solana.com/address/FmS7DSf1d5KGNQtVy6usXFaHi7fNnTtiJofHZSQzGKYi?cluster=devnet";

const LINKS = [
  { label: "Marketplace", href: "/marketplace", hint: "Browse and buy tracks" },
  { label: "Portfolio", href: "/portfolio", hint: "Your tokens and royalties" },
  { label: "Studio", href: "/studio", hint: "List a song" },
  { label: "How it works", href: "/#how", hint: "Three steps" },
];
// Shown only when the connected wallet is the on-chain verifier.
const VERIFIER_LINK = { label: "Verifier", href: "/verifier", hint: "Approve tracks" };

/** Fixed top navigation. Inline links on desktop, full-screen menu on mobile. */
export default function Menu() {
  const pathname = usePathname();
  const isVerifier = useIsVerifier();
  const [open, setOpen] = useState(false);

  const links = isVerifier ? [...LINKS.slice(0, 3), VERIFIER_LINK, LINKS[3]] : LINKS;

  useEffect(() => setOpen(false), [pathname]);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && setOpen(false);
    window.addEventListener("keydown", onKey);
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      window.removeEventListener("keydown", onKey);
      document.body.style.overflow = prev;
    };
  }, [open]);

  const active = (href: string) => !href.includes("#") && pathname.startsWith(href);

  return (
    <>
      <header className="fixed inset-x-0 top-0 z-50 border-b border-white/10 bg-ink/85 backdrop-blur-md">
        <div className="mx-auto flex h-[68px] max-w-[1600px] items-center justify-between gap-4 px-4 sm:h-[76px] sm:px-8">
          <Link href="/" className="text-2xl font-medium tracking-[-0.04em] sm:text-3xl">
            Assetra
          </Link>

          <nav className="hidden items-center gap-1 md:flex">
            {links.map((l) => (
              <Link
                key={l.href}
                href={l.href}
                className={`rounded-full px-4 py-2 text-base transition ${
                  active(l.href) ? "bg-white text-ink" : "text-white/75 hover:bg-white/10 hover:text-white"
                }`}
              >
                {l.label}
              </Link>
            ))}
          </nav>

          <div className="flex items-center gap-3">
            <ConnectButton />
            <button
              type="button"
              aria-label={open ? "Close menu" : "Open menu"}
              aria-expanded={open}
              onClick={() => setOpen((v) => !v)}
              className="flex h-11 w-11 flex-col items-center justify-center gap-[6px] bg-paper outline-none focus-visible:ring-4 focus-visible:ring-pop md:hidden"
            >
              {open ? (
                <>
                  <span className="h-[2px] w-6 translate-y-[4px] rotate-45 bg-ink" />
                  <span className="h-[2px] w-6 -translate-y-[4px] -rotate-45 bg-ink" />
                </>
              ) : (
                <>
                  <span className="h-[2px] w-6 bg-ink" />
                  <span className="h-[2px] w-6 bg-ink" />
                </>
              )}
            </button>
          </div>
        </div>
      </header>

      {open && (
        <div
          role="dialog"
          aria-modal="true"
          className="fixed inset-0 z-40 flex flex-col justify-between bg-paper px-4 pb-8 pt-28 text-ink md:hidden"
        >
          <nav className="flex flex-col">
            {links.map((l) => (
              <Link
                key={l.href}
                href={l.href}
                className="flex items-baseline justify-between border-b border-ink/25 py-4"
              >
                <span className="text-[clamp(2.25rem,9vw,4rem)] font-medium leading-none tracking-[-0.05em]">
                  {l.label}
                </span>
                <span className="text-sm">{l.hint}</span>
              </Link>
            ))}
          </nav>
          <a
            href={EXPLORER}
            target="_blank"
            rel="noreferrer"
            className="mt-10 w-fit rounded-full border border-ink px-5 py-2 text-base transition hover:bg-ink hover:text-paper"
          >
            Devnet program ↗
          </a>
        </div>
      )}
    </>
  );
}
