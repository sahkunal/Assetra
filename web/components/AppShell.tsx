import { ReactNode } from "react";
import Menu from "@/components/Menu";

export default function AppShell({
  title,
  subtitle,
  children,
}: {
  title: string;
  subtitle?: string;
  children: ReactNode;
}) {
  return (
    <div className="min-h-screen">
      <Menu />
      <main className="mx-auto max-w-6xl px-4 pb-24 pt-28 sm:px-8 sm:pt-36">
        <h1 className="text-[clamp(2.5rem,7vw,5.5rem)] font-medium leading-[0.95] tracking-[-0.05em]">
          {title}
        </h1>
        {subtitle && <p className="mt-4 max-w-2xl text-lg text-white/65">{subtitle}</p>}
        <div className="mt-10 sm:mt-14">{children}</div>
      </main>
    </div>
  );
}
