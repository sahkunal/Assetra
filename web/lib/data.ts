import { BN } from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import {
  AssetraProgram,
  ListingAccount,
  TrackAccount,
  TrackEntry,
  ata,
  statusOf,
} from "./program";

const toEntry = (r: { publicKey: PublicKey; account: unknown }): TrackEntry => {
  const account = r.account as TrackAccount;
  return { address: r.publicKey, account, status: statusOf(account) };
};

export async function loadTracks(
  program: AssetraProgram,
  filter?: { creator?: PublicKey }
): Promise<TrackEntry[]> {
  const filters = filter?.creator
    ? [{ memcmp: { offset: 8, bytes: filter.creator.toBase58() } }]
    : [];
  const rows = await (program.account as any).track.all(filters);
  return rows.map(toEntry).sort(
    (a: TrackEntry, b: TrackEntry) =>
      b.account.verifiedAt.cmp(a.account.verifiedAt) || a.account.title.localeCompare(b.account.title)
  );
}

export async function loadTrack(program: AssetraProgram, address: PublicKey) {
  const account = (await (program.account as any).track.fetchNullable(address)) as TrackAccount | null;
  return account ? { address, account, status: statusOf(account) } : null;
}

/** Token balance for a Token-2022 account; 0 if it does not exist. */
export async function tokenBalance(connection: Connection, account: PublicKey): Promise<bigint> {
  try {
    const r = await connection.getTokenAccountBalance(account);
    return BigInt(r.value.amount);
  } catch {
    return 0n;
  }
}

export const holderBalance = (connection: Connection, mint: PublicKey, owner: PublicKey) =>
  tokenBalance(connection, ata(owner, mint));

export async function loadListings(
  program: AssetraProgram,
  by: { track?: PublicKey; seller?: PublicKey }
): Promise<{ address: PublicKey; account: ListingAccount }[]> {
  const filters = [];
  if (by.track) filters.push({ memcmp: { offset: 8, bytes: by.track.toBase58() } });
  if (by.seller) filters.push({ memcmp: { offset: 40, bytes: by.seller.toBase58() } });
  const rows = await (program.account as any).listing.all(filters);
  return rows.map((r: any) => ({ address: r.publicKey, account: r.account as ListingAccount }));
}

export interface PositionView {
  track: TrackEntry;
  balance: bigint;
  accumulator: BN;
  rewardDebt: BN;
  unclaimed: BN;
  totalClaimed: BN;
}

export async function loadPositions(
  program: AssetraProgram,
  connection: Connection,
  holder: PublicKey
): Promise<PositionView[]> {
  const rows = await (program.account as any).holderPosition.all([
    { memcmp: { offset: 40, bytes: holder.toBase58() } },
  ]);

  const views = await Promise.all(
    rows.map(async (r: any): Promise<PositionView | null> => {
      const track = await loadTrack(program, r.account.track);
      if (!track || !track.account.mint) return null;
      const [balance, pool] = await Promise.all([
        holderBalance(connection, track.account.mint, holder),
        (program.account as any).revenuePool.fetchNullable(
          PublicKey.findProgramAddressSync(
            [Buffer.from("revenue_pool"), track.address.toBuffer()],
            program.programId
          )[0]
        ),
      ]);
      return {
        track,
        balance,
        accumulator: pool ? pool.accumulatedRewardsPerToken : new BN(0),
        rewardDebt: r.account.rewardDebt,
        unclaimed: r.account.unclaimedRewards,
        totalClaimed: r.account.totalClaimed,
      };
    })
  );
  return views.filter((v): v is PositionView => v !== null);
}
