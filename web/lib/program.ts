import { AnchorProvider, BN, Idl, Program } from "@coral-xyz/anchor";
import type { Wallet } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import idl from "./assetra.json";

export const PROGRAM_ID = new PublicKey((idl as { address: string }).address);
export const TOKEN_2022 = new PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
export const ATA_PROGRAM = new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

export const RPC_URL = process.env.NEXT_PUBLIC_RPC_URL || "https://api.devnet.solana.com";
export const CLUSTER = "devnet";

export const explorerAddress = (a: PublicKey | string) =>
  `https://explorer.solana.com/address/${a.toString()}?cluster=${CLUSTER}`;
export const explorerTx = (sig: string) =>
  `https://explorer.solana.com/tx/${sig}?cluster=${CLUSTER}`;

// ---------------------------------------------------------------- program --
const readOnlyKey = Keypair.generate().publicKey;
const readOnlyWallet = {
  publicKey: readOnlyKey,
  signTransaction: async () => {
    throw new Error("Connect a wallet to sign transactions");
  },
  signAllTransactions: async () => {
    throw new Error("Connect a wallet to sign transactions");
  },
} as unknown as Wallet;

export type AssetraProgram = Program<Idl>;

export function getProgram(connection: Connection, wallet?: Wallet | null): AssetraProgram {
  const provider = new AnchorProvider(connection, wallet ?? readOnlyWallet, {
    commitment: "confirmed",
  });
  return new Program(idl as Idl, provider);
}

// ------------------------------------------------------------------- PDAs --
const u64le = (n: BN | number) => new BN(n).toArrayLike(Buffer, "le", 8);
const find = (seeds: Buffer[]) => PublicKey.findProgramAddressSync(seeds, PROGRAM_ID)[0];

export const configPda = () => find([Buffer.from("config")]);
export const creatorProfilePda = (authority: PublicKey) =>
  find([Buffer.from("creator_profile"), authority.toBuffer()]);
export const trackPda = (creator: PublicKey, trackId: BN | number) =>
  find([Buffer.from("track"), creator.toBuffer(), u64le(trackId)]);
export const mintPda = (track: PublicKey) => find([Buffer.from("mint"), track.toBuffer()]);
export const vaultPda = (track: PublicKey) => find([Buffer.from("vault"), track.toBuffer()]);
export const revenuePoolPda = (track: PublicKey) =>
  find([Buffer.from("revenue_pool"), track.toBuffer()]);
export const holderPositionPda = (track: PublicKey, holder: PublicKey) =>
  find([Buffer.from("holder_position"), track.toBuffer(), holder.toBuffer()]);
export const listingPda = (track: PublicKey, seller: PublicKey, nonce: BN | number) =>
  find([Buffer.from("listing"), track.toBuffer(), seller.toBuffer(), u64le(nonce)]);

export const ata = (owner: PublicKey, mint: PublicKey) =>
  PublicKey.findProgramAddressSync(
    [owner.toBuffer(), TOKEN_2022.toBuffer(), mint.toBuffer()],
    ATA_PROGRAM
  )[0];

// ------------------------------------------------------------ account types --
export type TrackStatus = "configured" | "verified" | "minted";

export interface TrackAccount {
  creator: PublicKey;
  trackId: BN;
  title: string;
  isrc: string;
  coverArtUri: string;
  audioPreviewUri: string;
  declaredAnnualRevenue: BN;
  verifiedAnnualRevenue: BN;
  verifiedAt: BN;
  tokenizedBps: number;
  valuationMultipleBps: number;
  tokenSupply: BN;
  mint: PublicKey | null;
  vault: PublicKey | null;
  status: Record<string, unknown>;
}

export interface TrackEntry {
  address: PublicKey;
  account: TrackAccount;
  status: TrackStatus;
}

export const statusOf = (t: TrackAccount): TrackStatus =>
  Object.keys(t.status)[0] as TrackStatus;

export interface ListingAccount {
  track: PublicKey;
  seller: PublicKey;
  mint: PublicKey;
  pricePerToken: BN;
  amount: BN;
  nonce: BN;
}
