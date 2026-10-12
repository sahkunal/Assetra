/**
 * End-to-end rehearsal of the Assetra lifecycle against devnet.
 *
 *   register -> create track -> verify -> mint -> init pool -> buy
 *   -> deposit royalties -> claim -> list resale -> buy resale -> claim
 *
 * Needs `scripts/init-config.ts` to have run first (config + keys/verifier.json).
 * Uses a fresh creator / investor / buyer each run, funded from your CLI wallet
 * (about 0.4 SOL per run; the throwaway wallets are not saved, so that SOL
 * stays in them).
 *
 *   npx ts-node --transpile-only scripts/demo-flow.ts
 */
import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";

const RPC_URL = process.env.RPC_URL || "https://api.devnet.solana.com";
const WALLET_PATH =
  process.env.ANCHOR_WALLET ?? path.join(os.homedir(), ".config/solana/id.json");
const VERIFIER_PATH = path.join(__dirname, "..", "keys", "verifier.json");

const TOKEN_2022 = new PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const ATA_PROGRAM = new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

// Track parameters. Primary price per token =
//   declared * multiple_bps * tokenized_bps / (100 * 10_000 * supply) = 10_000 lamports
const DECLARED_REVENUE = new BN(1_000_000_000); // 1 SOL / year
const VERIFIED_REVENUE = new BN(1_000_000_000);
const TOKENIZED_BPS = 2_000; // 20% of royalties
const MULTIPLE_BPS = 500;
const TOKEN_SUPPLY = new BN(100_000);

const BUY_AMOUNT = new BN(1_000); // 1% of supply
const DEPOSIT = new BN(100_000_000); // 0.1 SOL of royalties
const RESALE_AMOUNT = new BN(400);
const RESALE_PRICE = new BN(20_000); // lamports per token
const NONCE = new BN(0);

const u64 = (n: BN | number) => new BN(n).toArrayLike(Buffer, "le", 8);
const pda = (seeds: Buffer[], programId: PublicKey) =>
  PublicKey.findProgramAddressSync(seeds, programId)[0];
const ata = (owner: PublicKey, mint: PublicKey) =>
  PublicKey.findProgramAddressSync(
    [owner.toBuffer(), TOKEN_2022.toBuffer(), mint.toBuffer()],
    ATA_PROGRAM
  )[0];

function loadKeypair(file: string): Keypair {
  return Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(file, "utf8"))));
}

const sol = (lamports: number | BN) => (Number(lamports.toString()) / LAMPORTS_PER_SOL).toFixed(6);

async function main() {
  const payer = loadKeypair(WALLET_PATH);
  if (!fs.existsSync(VERIFIER_PATH)) {
    throw new Error("keys/verifier.json not found — run scripts/init-config.ts first");
  }
  const verifier = loadKeypair(VERIFIER_PATH);

  const connection = new anchor.web3.Connection(RPC_URL, "confirmed");
  const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(payer), {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);
  const idl = JSON.parse(
    fs.readFileSync(path.join(__dirname, "..", "idls", "assetra.json"), "utf8")
  );
  const program = new anchor.Program(idl, provider);
  const pid = program.programId;
  const acct = program.account as any;

  const configPda = pda([Buffer.from("config")], pid);
  if (!(await connection.getAccountInfo(configPda))) {
    throw new Error("Config not initialized — run scripts/init-config.ts first");
  }
  const config = await acct.programConfig.fetch(configPda);
  if (!config.verifier.equals(verifier.publicKey)) {
    throw new Error("keys/verifier.json is not the verifier set in the on-chain config");
  }

  const creator = Keypair.generate();
  const investor = Keypair.generate();
  const buyer = Keypair.generate();

  const step = (n: number, msg: string) => console.log(`\n[${n}] ${msg}`);
  const done = (sig: string) => console.log(`    tx ${sig}`);

  // ---- fund actors ---------------------------------------------------------
  step(0, "Funding creator, investor and buyer from your CLI wallet");
  const fundTx = new Transaction();
  for (const [kp, amount] of [
    [creator, 0.3],
    [investor, 0.05],
    [buyer, 0.05],
  ] as [Keypair, number][]) {
    fundTx.add(
      SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: kp.publicKey,
        lamports: Math.round(amount * LAMPORTS_PER_SOL),
      })
    );
  }
  done(await provider.sendAndConfirm(fundTx));
  console.log("    creator :", creator.publicKey.toBase58());
  console.log("    investor:", investor.publicKey.toBase58());
  console.log("    buyer   :", buyer.publicKey.toBase58());

  // ---- derived addresses ---------------------------------------------------
  const creatorProfile = pda([Buffer.from("creator_profile"), creator.publicKey.toBuffer()], pid);
  const track = pda([Buffer.from("track"), creator.publicKey.toBuffer(), u64(0)], pid);
  const mint = pda([Buffer.from("mint"), track.toBuffer()], pid);
  const vault = pda([Buffer.from("vault"), track.toBuffer()], pid);
  const revenuePool = pda([Buffer.from("revenue_pool"), track.toBuffer()], pid);
  const position = (holder: PublicKey) =>
    pda([Buffer.from("holder_position"), track.toBuffer(), holder.toBuffer()], pid);
  const listing = pda(
    [Buffer.from("listing"), track.toBuffer(), investor.publicKey.toBuffer(), u64(NONCE)],
    pid
  );

  // ---- 1. creator onboarding -----------------------------------------------
  step(1, "Creator registers");
  done(
    await program.methods
      .registerCreator("Demo Artist", "I own the rights to Midnight Drive.")
      .accounts({ authority: creator.publicKey, creatorProfile })
      .signers([creator])
      .rpc()
  );

  step(2, "Creator declares a track and its tokenization terms");
  done(
    await program.methods
      .createTrack(
        "Midnight Drive",
        "USRC17600001",
        "https://example.com/cover.png",
        "https://example.com/preview.mp3",
        DECLARED_REVENUE,
        TOKENIZED_BPS,
        MULTIPLE_BPS,
        TOKEN_SUPPLY
      )
      .accounts({ authority: creator.publicKey, creatorProfile, track })
      .signers([creator])
      .rpc()
  );

  // ---- 3. verification -----------------------------------------------------
  step(3, "Verifier attests the track's royalty data");
  done(
    await program.methods
      .verifyTrack(VERIFIED_REVENUE)
      .accounts({ verifier: verifier.publicKey, config: configPda, track })
      .signers([verifier])
      .rpc()
  );

  // ---- 4. tokenization -----------------------------------------------------
  step(4, "Creator mints the Token-2022 supply into the vault");
  done(
    await program.methods
      .mintTrackTokens()
      .accounts({
        authority: creator.publicKey,
        track,
        mint,
        vault,
        tokenProgram: TOKEN_2022,
      })
      .signers([creator])
      .rpc()
  );

  step(5, "Creator opens the revenue pool");
  done(
    await program.methods
      .initializeRevenuePool()
      .accounts({ authority: creator.publicKey, track, revenuePool })
      .signers([creator])
      .rpc()
  );

  // ---- 6. primary sale -----------------------------------------------------
  step(6, `Investor buys ${BUY_AMOUNT} tokens from the vault`);
  const creatorBefore = await connection.getBalance(creator.publicKey);
  done(
    await program.methods
      .buyTokens(BUY_AMOUNT)
      .accounts({
        investor: investor.publicKey,
        creator: creator.publicKey,
        track,
        vault,
        investorTokenAccount: ata(investor.publicKey, mint),
        mint,
        revenuePool,
        holderPosition: position(investor.publicKey),
        tokenProgram: TOKEN_2022,
      })
      .signers([investor])
      .rpc()
  );
  const creatorAfter = await connection.getBalance(creator.publicKey);
  console.log(`    creator received ${sol(creatorAfter - creatorBefore)} SOL`);

  // ---- 7. royalties --------------------------------------------------------
  step(7, `Creator deposits ${sol(DEPOSIT)} SOL of royalties`);
  done(
    await program.methods
      .depositRevenue(DEPOSIT)
      .accounts({ authority: creator.publicKey, track, revenuePool })
      .signers([creator])
      .rpc()
  );

  // ---- 8. claim ------------------------------------------------------------
  const claim = async (label: string, holder: Keypair) => {
    const before = await connection.getBalance(holder.publicKey);
    done(
      await program.methods
        .claimRevenue()
        .accounts({
          holder: holder.publicKey,
          track,
          holderTokenAccount: ata(holder.publicKey, mint),
          revenuePool,
          holderPosition: position(holder.publicKey),
        })
        .signers([holder])
        .rpc()
    );
    const pos = await acct.holderPosition.fetch(position(holder.publicKey));
    console.log(`    ${label} total claimed: ${sol(pos.totalClaimed)} SOL`);
    return before;
  };
  step(8, "Investor claims their pro-rata share (expect 1% = 0.001 SOL)");
  await claim("investor", investor);

  // ---- 9. secondary market -------------------------------------------------
  step(9, `Investor lists ${RESALE_AMOUNT} tokens at ${RESALE_PRICE} lamports each`);
  done(
    await program.methods
      .listResale(RESALE_AMOUNT, RESALE_PRICE, NONCE)
      .accounts({
        seller: investor.publicKey,
        track,
        sellerTokenAccount: ata(investor.publicKey, mint),
        mint,
        listing,
        tokenProgram: TOKEN_2022,
      })
      .signers([investor])
      .rpc()
  );

  step(10, "Buyer purchases the listing");
  done(
    await program.methods
      .buyResale()
      .accounts({
        buyer: buyer.publicKey,
        seller: investor.publicKey,
        track,
        sellerTokenAccount: ata(investor.publicKey, mint),
        buyerTokenAccount: ata(buyer.publicKey, mint),
        mint,
        revenuePool,
        sellerHolderPosition: position(investor.publicKey),
        buyerHolderPosition: position(buyer.publicKey),
        listing,
        tokenProgram: TOKEN_2022,
      })
      .signers([buyer])
      .rpc()
  );

  step(11, "Second royalty deposit — now split between investor (600) and buyer (400)");
  done(
    await program.methods
      .depositRevenue(DEPOSIT)
      .accounts({ authority: creator.publicKey, track, revenuePool })
      .signers([creator])
      .rpc()
  );

  step(12, "Investor and buyer claim (expect 0.0006 and 0.0004 SOL)");
  await claim("investor", investor);
  await claim("buyer   ", buyer);

  // ---- summary -------------------------------------------------------------
  const t = await acct.track.fetch(track);
  console.log("\n================ SUMMARY ================");
  console.log("Track      :", track.toBase58());
  console.log("Mint       :", mint.toBase58());
  console.log("Status     :", Object.keys(t.status)[0]);
  console.log("Verified at:", new Date(Number(t.verifiedAt) * 1000).toISOString());
  console.log(`Explorer   : https://explorer.solana.com/address/${track.toBase58()}?cluster=devnet`);
  console.log("=========================================");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
