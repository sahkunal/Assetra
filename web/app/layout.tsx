import type { Metadata } from "next";
import { Outfit } from "next/font/google";
import Providers from "@/components/Providers";
import "./globals.css";

const display = Outfit({ subsets: ["latin"], variable: "--font-display" });

export const metadata: Metadata = {
  title: "Assetra — own a slice of the song",
  description:
    "Tokenize a share of a song's future royalty income on Solana. Verified tracks, fractional tokens, royalties paid on-chain to holders.",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body className={`${display.variable} font-display antialiased`}><Providers>{children}</Providers></body>
    </html>
  );
}
