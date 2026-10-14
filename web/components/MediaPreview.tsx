"use client";

import { resolveUri, youtubeId } from "@/components/Cover";

/** Plays a preview: YouTube embed, or a native audio player for file links. */
export default function MediaPreview({ uri, title }: { uri: string; title: string }) {
  const yt = youtubeId(uri);
  const src = resolveUri(uri);

  if (yt) {
    return (
      <iframe
        title={`${title} preview`}
        src={`https://www.youtube-nocookie.com/embed/${yt}`}
        allow="encrypted-media; picture-in-picture"
        allowFullScreen
        className="aspect-video w-full border border-white/15"
      />
    );
  }
  if (src) {
    // eslint-disable-next-line jsx-a11y/media-has-caption
    return <audio controls preload="none" src={src} className="w-full" />;
  }
  return <p className="text-white/50">No preview provided.</p>;
}
