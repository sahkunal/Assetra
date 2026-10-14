"use client";

import { useEffect, useState } from "react";

/** Accept http(s) and ipfs:// only; route ipfs:// through a public gateway. */
export function resolveUri(uri: string | undefined | null): string | null {
  const u = (uri ?? "").trim();
  if (!u) return null;
  if (u.startsWith("ipfs://")) return `https://ipfs.io/ipfs/${u.slice(7).replace(/^ipfs\//, "")}`;
  if (/^https?:\/\//i.test(u)) return u;
  return null;
}

/** Extract a YouTube video id from youtu.be / youtube.com URLs. */
export function youtubeId(uri: string | undefined | null): string | null {
  const u = (uri ?? "").trim();
  if (!/^https?:\/\//i.test(u)) return null;
  try {
    const url = new URL(u);
    const host = url.hostname.replace(/^www\./, "").replace(/^m\./, "");
    let id: string | null = null;
    if (host === "youtu.be") id = url.pathname.slice(1).split("/")[0];
    else if (host === "youtube.com" || host === "music.youtube.com") {
      if (url.pathname === "/watch") id = url.searchParams.get("v");
      else if (url.pathname.startsWith("/embed/") || url.pathname.startsWith("/shorts/"))
        id = url.pathname.split("/")[2];
    }
    return id && /^[\w-]{6,20}$/.test(id) ? id : null;
  } catch {
    return null;
  }
}

/**
 * Track cover art. Falls back to a plain tile with the title's initial when
 * the URL is missing, unsupported, or does not load as an image.
 */
export default function Cover({
  uri,
  title,
  className = "",
}: {
  uri: string | undefined | null;
  title: string;
  className?: string;
}) {
  const src = resolveUri(uri);
  const [failed, setFailed] = useState(false);
  useEffect(() => setFailed(false), [src]);

  if (!src || failed) {
    return (
      <div
        aria-hidden
        className={`flex items-center justify-center bg-white/[0.06] text-white/40 ${className}`}
      >
        <span className="text-5xl font-medium tracking-[-0.05em]">
          {title.trim().charAt(0).toUpperCase() || "♪"}
        </span>
      </div>
    );
  }

  return (
    // Plain <img>: cover URLs come from arbitrary hosts chosen by creators.
    // eslint-disable-next-line @next/next/no-img-element
    <img
      src={src}
      alt={`${title} cover art`}
      loading="lazy"
      referrerPolicy="no-referrer"
      onError={() => setFailed(true)}
      className={`object-cover ${className}`}
    />
  );
}
