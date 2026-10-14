import Arcs from "@/components/Arcs";
import Logo from "@/components/Logo";
import Menu from "@/components/Menu";
import ShowcaseCards from "@/components/ShowcaseCards";

const PROGRAM_ID = "FmS7DSf1d5KGNQtVy6usXFaHi7fNnTtiJofHZSQzGKYi";
const EXPLORER = `https://explorer.solana.com/address/${PROGRAM_ID}?cluster=devnet`;

const STEPS = [
  {
    n: "01",
    title: "Artists list a slice",
    body: "A rights holder tokenizes a share of a song's future royalty income — not the audio, not the copyright.",
  },
  {
    n: "02",
    title: "Verified, then sold",
    body: "A verifier attests the royalty data on-chain before a single token can be minted or sold.",
  },
  {
    n: "03",
    title: "Royalties flow to holders",
    body: "Deposits are split pro-rata to whoever holds the tokens, and holders can resell any time.",
  },
];

export default function Home() {
  return (
    <main>
      {/* ------------------------------------------------------------ hero */}
      <section className="relative min-h-screen overflow-hidden">
        <Arcs />

        <Menu />

        {/* live-track showcase, right side of the hero (desktop only) */}
        <div className="pointer-events-none absolute right-6 top-[84px] z-[1] hidden h-[calc(100%-300px)] w-[46%] items-center lg:flex xl:right-14">
          <div className="pointer-events-auto w-full">
            <ShowcaseCards />
          </div>
        </div>

        <div className="relative z-[1] flex min-h-screen flex-col justify-end px-4 pb-6 sm:px-8 sm:pb-10">
          <p className="mb-5 text-sm uppercase tracking-[0.2em] text-white/60">
            Music royalties, tokenized on Solana
          </p>

          <h1 className="text-[clamp(3.25rem,10.5vw,10rem)] font-medium leading-[0.92] tracking-[-0.05em]">
            Own a slice
            <br />
            of the song
          </h1>

          {/* bottom bar */}
          <div className="mt-10 flex w-full max-w-[1080px] flex-col bg-paper text-ink sm:mt-14 sm:flex-row sm:items-stretch">
            <div className="flex items-center gap-4 border-b border-ink/15 p-3 sm:border-b-0 sm:border-r sm:p-4">
              <Logo />
              <span className="text-3xl font-medium tracking-[-0.04em] sm:text-4xl">Assetra</span>
            </div>

            <p className="flex flex-1 items-center border-b border-ink/15 px-5 py-4 text-base leading-snug sm:border-b-0 sm:border-r sm:text-lg">
              Fractional royalty tokens for independent artists. Verified on-chain, paid out to holders.
            </p>

            <div className="flex items-center justify-between gap-3 p-3 sm:p-4">
              <a
                href={EXPLORER}
                target="_blank"
                rel="noreferrer"
                className="rounded-full border border-ink px-5 py-2 text-base transition hover:bg-ink hover:text-paper"
              >
                Devnet
              </a>
              <a
                href="/marketplace"
                className="pixel-cut bg-pop px-7 py-3 text-lg font-medium tracking-[-0.02em] text-ink transition hover:brightness-110"
              >
                Launch app
              </a>
            </div>
          </div>
        </div>
      </section>

      {/* --------------------------------------------------------- statement */}
      <section className="columns relative overflow-hidden bg-paper px-4 py-24 text-ink sm:px-8 sm:py-36">
        <h2 className="text-[clamp(2.75rem,8.4vw,8rem)] font-medium leading-[0.95] tracking-[-0.05em]">
          Royalties are locked up.
          <br />
          <span className="block sm:ml-[16.6%]">Let fans own a piece.</span>
        </h2>
        <p className="mt-10 max-w-xl text-lg leading-snug sm:ml-[16.6%] sm:text-xl">
          Artists earn from a song for years but can&apos;t access that future income today.
          Assetra turns a defined slice of it into tokens anyone can hold.
        </p>
      </section>

      {/* ----------------------------------------------------- how it works */}
      <section id="how" className="px-4 py-24 sm:px-8 sm:py-32">
        <p className="mb-6 w-fit rounded-full border border-white/40 px-4 py-1 text-base">How it works</p>

        <h2 className="max-w-[16ch] text-[clamp(2.75rem,8vw,7rem)] font-medium leading-[0.95] tracking-[-0.05em] [text-wrap:balance]">
          Three steps from a song to a share.
        </h2>

        <div className="mt-14 grid border-y border-white/20 sm:mt-20 md:grid-cols-3">
          {STEPS.map((s, i) => (
            <div
              key={s.n}
              className={`p-6 sm:p-8 ${i > 0 ? "border-t border-white/20 md:border-l md:border-t-0" : ""}`}
            >
              <span className="text-sm tracking-[0.2em] text-pop">{s.n}</span>
              <h3 className="mt-8 text-3xl font-medium leading-[1] tracking-[-0.04em] sm:text-4xl">{s.title}</h3>
              <p className="mt-4 text-lg leading-snug text-white/65">{s.body}</p>
            </div>
          ))}
        </div>
      </section>

      {/* ---------------------------------------------------------- audiences */}
      <section className="bg-paper px-4 py-24 text-ink sm:px-8 sm:py-32">
        <div className="grid gap-4 md:grid-cols-2">
          <a
            href="/studio"
            className="flex min-h-[18rem] flex-col justify-between bg-ink p-6 text-white transition hover:brightness-125 sm:p-8"
          >
            <span className="text-sm uppercase tracking-[0.2em] text-pop">For artists</span>
            <span>
              <span className="block text-4xl font-medium leading-[1] tracking-[-0.04em] sm:text-5xl">
                Raise from your catalog without selling it.
              </span>
              <span className="mt-6 inline-block text-lg underline underline-offset-4">Open the studio →</span>
            </span>
          </a>

          <a
            href="/marketplace"
            className="flex min-h-[18rem] flex-col justify-between border border-ink p-6 transition hover:bg-ink hover:text-white sm:p-8"
          >
            <span className="text-sm uppercase tracking-[0.2em]">For investors</span>
            <span>
              <span className="block text-4xl font-medium leading-[1] tracking-[-0.04em] sm:text-5xl">
                Back songs you believe in. Earn as they play.
              </span>
              <span className="mt-6 inline-block text-lg underline underline-offset-4">Browse tracks →</span>
            </span>
          </a>
        </div>
      </section>

      {/* ------------------------------------------------------------- proof */}
      <section className="px-4 py-24 sm:px-8 sm:py-32">
        <h2 className="max-w-[14ch] text-[clamp(2.75rem,8vw,7rem)] font-medium leading-[0.95] tracking-[-0.05em] [text-wrap:balance]">
          Don&apos;t trust us. Check the chain.
        </h2>

        <div className="mt-12 grid gap-px bg-white/20 sm:mt-16 md:max-w-3xl">
          <div className="bg-ink p-6 sm:p-8">
            <p className="text-2xl leading-snug sm:text-3xl">
              Every instruction is exercised against the deployed program — 36 integration tests on
              devnet and 37 local tests, all passing.
            </p>
          </div>
          <div className="flex flex-wrap items-center justify-between gap-4 bg-ink p-4 sm:p-5">
            <div>
              <p className="text-lg font-medium leading-tight">Assetra program · devnet</p>
              <p className="break-all text-sm text-white/55">{PROGRAM_ID}</p>
            </div>
            <a
              href={EXPLORER}
              target="_blank"
              rel="noreferrer"
              className="pixel-cut bg-pop px-6 py-3 text-lg font-medium tracking-[-0.02em] text-ink transition hover:brightness-110"
            >
              View on Explorer
            </a>
          </div>
        </div>

        <dl className="mt-14 grid gap-8 sm:mt-20 sm:grid-cols-3">
          {[
            ["Verified first", "No track can be minted or sold until a verifier signs off on its royalty data."],
            ["Token-2022 shares", "Each track is its own fixed-supply token. One token, one proportional claim."],
            ["Paid on-chain", "Royalty deposits are split pro-rata to current holders. Claim any time."],
          ].map(([t, d]) => (
            <div key={t} className="border-t border-white/20 pt-4">
              <dt className="text-xl font-medium tracking-[-0.02em]">{t}</dt>
              <dd className="mt-2 leading-snug text-white/60">{d}</dd>
            </div>
          ))}
        </dl>
      </section>

      {/* ------------------------------------------------------------ footer */}
      <footer className="flex flex-col gap-3 border-t border-white/10 px-4 py-8 text-sm text-white/55 sm:flex-row sm:items-center sm:justify-between sm:px-8">
        <span>© Assetra — capstone project, running on Solana devnet.</span>
        <a href={EXPLORER} target="_blank" rel="noreferrer" className="break-all underline underline-offset-4 hover:text-white">
          Program {PROGRAM_ID}
        </a>
      </footer>
    </main>
  );
}
