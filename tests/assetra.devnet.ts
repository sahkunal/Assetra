/**
 * Assetra — integration tests against a live cluster (devnet).
 *
 * These drive the deployed program through the real RPC, so they exercise the
 * actual Token-2022 program, fees, rent and clock. They complement the fast
 * in-process LiteSVM suite in programs/assetra/tests.
 *
 * Prerequisites
 *   - program deployed at the ID in idls/assetra.json
 *   - `scripts/init-config.ts` has run (config exists, keys/verifier.json)
 *
 * Run
 *   RPC_URL=<your devnet rpc> yarn test:devnet
 *
 * Every run uses fresh creator / investor / buyer wallets (funded from the CLI
 * wallet, ~0.3 SOL per run) so it can be repeated. Tests inside a describe
 * block build on each other in order, mirroring the product lifecycle.
 */
import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
import {
  createAssociatedTokenAccount,
  createMint,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import { expect } from "chai";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------
const RPC_URL =
  process.env.RPC_URL || process.env.ANCHOR_PROVIDER_URL || "https://api.devnet.solana.com";
if (!/^https?:\/\//.test(RPC_URL)) {
  throw new Error(
    `RPC_URL must start with http:// or https:// (got "${RPC_URL}"). ` +
      "Unset it to use the public devnet RPC, or set it to a real endpoint."
  );
}
const WALLET_PATH =
  process.env.ANCHOR_WALLET ?? path.join(os.homedir(), ".config/solana/id.json");
const VERIFIER_PATH = path.join(__dirname, "..", "keys", "verifier.json");

const TOKEN_2022 = new PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const ATA_PROGRAM = new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

// price per token = declared * multiple_bps * tokenized_bps / (100 * 10_000 * supply)
const DECLARED_REVENUE = new BN(100_000_000);
const TOKENIZED_BPS = 2_000;
const MULTIPLE_BPS = 500;
const TOKEN_SUPPLY = 100_000;
const PRICE_PER_TOKEN = 1_000; // lamports, derived from the numbers above
const BUY_AMOUNT = 1_000; // 1% of supply
const DEPOSIT = 10_000_000; // lamports; 1% of it = 100_000

const loadKeypair = (file: string) =>
  Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(file, "utf8"))));
const u64 = (n: number | BN) => new BN(n).toArrayLike(Buffer, "le", 8);
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

const payer = loadKeypair(WALLET_PATH);
const connection = new anchor.web3.Connection(RPC_URL, "confirmed");
const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(payer), {
  commitment: "confirmed",
});
anchor.setProvider(provider);
const idl = JSON.parse(
  fs.readFileSync(path.join(__dirname, "..", "idls", "assetra.json"), "utf8")
);
const program = new anchor.Program(idl, provider);
const acct = program.account as any;
const pid = program.programId;

const pda = (seeds: Buffer[]) => PublicKey.findProgramAddressSync(seeds, pid)[0];
const ata = (owner: PublicKey, mint: PublicKey) =>
  PublicKey.findProgramAddressSync(
    [owner.toBuffer(), TOKEN_2022.toBuffer(), mint.toBuffer()],
    ATA_PROGRAM
  )[0];

const configPda = pda([Buffer.from("config")]);

/** Asserts that `p` rejects, optionally with an error naming `code`. */
async function expectFail(p: Promise<unknown>, code?: string) {
  try {
    await p;
  } catch (e: any) {
    if (!code) return;
    const haystack = [
      e?.error?.errorCode?.code,
      e?.error?.errorMessage,
      e?.message,
      Array.isArray(e?.logs) ? e.logs.join("\n") : "",
    ].join("\n");
    expect(haystack, `expected error ${code}`).to.include(code);
    return;
  }
  expect.fail(`expected the transaction to fail${code ? ` with ${code}` : ""}`);
}

const tokenBalance = async (addr: PublicKey) =>
  Number((await connection.getTokenAccountBalance(addr)).value.amount);

// ---------------------------------------------------------------------------
// Shared fixture: one track that moves through the whole lifecycle
// ---------------------------------------------------------------------------
const creator = Keypair.generate();
const investor = Keypair.generate();
const lateInvestor = Keypair.generate();
const buyer = Keypair.generate();
const outsider = Keypair.generate();
let verifier: Keypair;

const creatorProfile = pda([Buffer.from("creator_profile"), creator.publicKey.toBuffer()]);
const track = pda([Buffer.from("track"), creator.publicKey.toBuffer(), u64(0)]);
const mint = pda([Buffer.from("mint"), track.toBuffer()]);
const vault = pda([Buffer.from("vault"), track.toBuffer()]);
const revenuePool = pda([Buffer.from("revenue_pool"), track.toBuffer()]);
const position = (holder: PublicKey) =>
  pda([Buffer.from("holder_position"), track.toBuffer(), holder.toBuffer()]);
const listingPda = (seller: PublicKey, nonce: number) =>
  pda([Buffer.from("listing"), track.toBuffer(), seller.toBuffer(), u64(nonce)]);

const createTrack = (
  authority: Keypair,
  overrides: { tokenizedBps?: number; track?: PublicKey; profile?: PublicKey } = {}
) =>
  program.methods
    .createTrack(
      "Midnight Drive",
      "USRC17600001",
      "https://example.com/cover.png",
      "https://example.com/preview.mp3",
      DECLARED_REVENUE,
      overrides.tokenizedBps ?? TOKENIZED_BPS,
      MULTIPLE_BPS,
      new BN(TOKEN_SUPPLY)
    )
    .accounts({
      authority: authority.publicKey,
      creatorProfile: overrides.profile ?? creatorProfile,
      track: overrides.track ?? track,
    })
    .signers([authority]);

const verifyTrack = (signer: Keypair, revenue: number) =>
  program.methods
    .verifyTrack(new BN(revenue))
    .accounts({ verifier: signer.publicKey, config: configPda, track })
    .signers([signer]);

const mintTokens = () =>
  program.methods
    .mintTrackTokens()
    .accounts({
      authority: creator.publicKey,
      track,
      mint,
      vault,
      tokenProgram: TOKEN_2022,
    })
    .signers([creator]);

const buyTokens = (who: Keypair, amount: number) =>
  program.methods
    .buyTokens(new BN(amount))
    .accounts({
      investor: who.publicKey,
      creator: creator.publicKey,
      track,
      vault,
      investorTokenAccount: ata(who.publicKey, mint),
      mint,
      revenuePool,
      holderPosition: position(who.publicKey),
      tokenProgram: TOKEN_2022,
    })
    .signers([who]);

const depositRevenue = (who: Keypair, amount: number) =>
  program.methods
    .depositRevenue(new BN(amount))
    .accounts({ authority: who.publicKey, track, revenuePool })
    .signers([who]);

const claimRevenue = (who: Keypair) =>
  program.methods
    .claimRevenue()
    .accounts({
      holder: who.publicKey,
      track,
      holderTokenAccount: ata(who.publicKey, mint),
      revenuePool,
      holderPosition: position(who.publicKey),
    })
    .signers([who]);

const listResale = (seller: Keypair, amount: number, price: number, nonce: number) =>
  program.methods
    .listResale(new BN(amount), new BN(price), new BN(nonce))
    .accounts({
      seller: seller.publicKey,
      track,
      sellerTokenAccount: ata(seller.publicKey, mint),
      mint,
      listing: listingPda(seller.publicKey, nonce),
      tokenProgram: TOKEN_2022,
    })
    .signers([seller]);

const cancelResale = (signer: Keypair, seller: PublicKey, nonce: number) =>
  program.methods
    .cancelResale()
    .accounts({
      seller: signer.publicKey,
      track,
      sellerTokenAccount: ata(signer.publicKey, mint),
      listing: listingPda(seller, nonce),
      tokenProgram: TOKEN_2022,
    })
    .signers([signer]);

const buyResale = (who: Keypair, seller: PublicKey, nonce: number) =>
  program.methods
    .buyResale()
    .accounts({
      buyer: who.publicKey,
      seller,
      track,
      sellerTokenAccount: ata(seller, mint),
      buyerTokenAccount: ata(who.publicKey, mint),
      mint,
      revenuePool,
      sellerHolderPosition: position(seller),
      buyerHolderPosition: position(who.publicKey),
      listing: listingPda(seller, nonce),
      tokenProgram: TOKEN_2022,
    })
    .signers([who]);

// ---------------------------------------------------------------------------
describe("assetra (devnet)", function () {
  this.timeout(600_000);

  // Gentle on the public RPC.
  afterEach(() => sleep(400));

  before(async () => {
    expect(fs.existsSync(VERIFIER_PATH), "keys/verifier.json missing — run scripts/init-config.ts").to.be
      .true;
    verifier = loadKeypair(VERIFIER_PATH);

    const deployed = await connection.getAccountInfo(pid);
    expect(deployed, `program ${pid.toBase58()} is not deployed on this cluster`).to.not.be.null;

    const config = await acct.programConfig.fetch(configPda);
    expect(config.verifier.toBase58(), "keys/verifier.json is not the on-chain verifier").to.equal(
      verifier.publicKey.toBase58()
    );

    // Fund the throwaway actors (they pay rent for the accounts they create).
    const fund = new Transaction();
    for (const [kp, sol] of [
      [creator, 0.08],
      [investor, 0.03],
      [lateInvestor, 0.03],
      [buyer, 0.03],
    ] as [Keypair, number][]) {
      fund.add(
        SystemProgram.transfer({
          fromPubkey: payer.publicKey,
          toPubkey: kp.publicKey,
          lamports: Math.round(sol * LAMPORTS_PER_SOL),
        })
      );
    }
    await provider.sendAndConfirm(fund);
  });

  // -------------------------------------------------------------------------
  describe("config & verifier", () => {
    it("config holds the expected admin and verifier", async () => {
      const config = await acct.programConfig.fetch(configPda);
      expect(config.verifier.toBase58()).to.equal(verifier.publicKey.toBase58());
      expect(config.admin.toBase58()).to.equal(payer.publicKey.toBase58());
    });

    it("initialize_config cannot be run a second time", async () => {
      await expectFail(
        program.methods
          .initializeConfig(outsider.publicKey)
          .accounts({ admin: outsider.publicKey })
          .signers([outsider])
          .rpc()
      );
    });

    it("set_verifier rejects a non-admin", async () => {
      await expectFail(
        program.methods
          .setVerifier(outsider.publicKey)
          .accounts({ admin: outsider.publicKey, config: configPda })
          .signers([outsider])
          .rpc(),
        "Unauthorized"
      );
    });
  });

  // -------------------------------------------------------------------------
  describe("creator onboarding", () => {
    it("register_creator rejects an oversized display name", async () => {
      await expectFail(
        program.methods
          .registerCreator("x".repeat(65), "I own the rights.")
          .accounts({ authority: creator.publicKey, creatorProfile })
          .signers([creator])
          .rpc(),
        "DisplayNameTooLong"
      );
    });

    it("register_creator creates a profile", async () => {
      await program.methods
        .registerCreator("Demo Artist", "I own the rights to Midnight Drive.")
        .accounts({ authority: creator.publicKey, creatorProfile })
        .signers([creator])
        .rpc();

      const p = await acct.creatorProfile.fetch(creatorProfile);
      expect(p.authority.toBase58()).to.equal(creator.publicKey.toBase58());
      expect(p.displayName).to.equal("Demo Artist");
      expect(p.selfAttested).to.be.true;
      expect(p.adminVerified).to.be.false;
      expect(p.trackCount.toNumber()).to.equal(0);
    });

    it("create_track rejects zero tokenized_bps", async () => {
      await expectFail(createTrack(creator, { tokenizedBps: 0 }).rpc(), "InvalidTokenizedBps");
    });

    it("create_track stores metadata and tokenization terms", async () => {
      await createTrack(creator).rpc();

      const t = await acct.track.fetch(track);
      expect(t.creator.toBase58()).to.equal(creator.publicKey.toBase58());
      expect(t.title).to.equal("Midnight Drive");
      expect(t.isrc).to.equal("USRC17600001");
      expect(t.tokenSupply.toNumber()).to.equal(TOKEN_SUPPLY);
      expect(Object.keys(t.status)[0]).to.equal("configured");
      expect(t.mint).to.be.null;

      const p = await acct.creatorProfile.fetch(creatorProfile);
      expect(p.trackCount.toNumber()).to.equal(1);
    });
  });

  // -------------------------------------------------------------------------
  describe("verification gate", () => {
    it("mint_track_tokens rejects an unverified track", async () => {
      await expectFail(mintTokens().rpc(), "InvalidTrackStatus");
    });

    it("verify_track rejects the creator (not the verifier)", async () => {
      await expectFail(verifyTrack(creator, 100_000_000).rpc(), "Unauthorized");
    });

    it("verify_track rejects zero verified revenue", async () => {
      await expectFail(verifyTrack(verifier, 0).rpc(), "InvalidVerifiedRevenue");
    });

    it("verify_track moves the track to Verified and records the revenue", async () => {
      await verifyTrack(verifier, 100_000_000).rpc();

      const t = await acct.track.fetch(track);
      expect(Object.keys(t.status)[0]).to.equal("verified");
      expect(t.verifiedAnnualRevenue.toNumber()).to.equal(100_000_000);
      expect(t.verifiedAt.toNumber()).to.be.greaterThan(0);
    });

    it("verify_track cannot be applied twice", async () => {
      await expectFail(verifyTrack(verifier, 200_000_000).rpc(), "InvalidTrackStatus");
    });
  });

  // -------------------------------------------------------------------------
  describe("tokenization", () => {
    it("mint_track_tokens mints the full supply into the vault", async () => {
      await mintTokens().rpc();

      const t = await acct.track.fetch(track);
      expect(Object.keys(t.status)[0]).to.equal("minted");
      expect(t.mint.toBase58()).to.equal(mint.toBase58());
      expect(t.vault.toBase58()).to.equal(vault.toBase58());
      expect(await tokenBalance(vault)).to.equal(TOKEN_SUPPLY);
    });

    it("mint_track_tokens cannot mint a second time", async () => {
      await expectFail(mintTokens().rpc());
    });

    it("initialize_revenue_pool starts at zero", async () => {
      await program.methods
        .initializeRevenuePool()
        .accounts({ authority: creator.publicKey, track, revenuePool })
        .signers([creator])
        .rpc();

      const pool = await acct.revenuePool.fetch(revenuePool);
      expect(pool.track.toBase58()).to.equal(track.toBase58());
      expect(pool.totalDeposited.toNumber()).to.equal(0);
      expect(pool.accumulatedRewardsPerToken.toString()).to.equal("0");
    });
  });

  // -------------------------------------------------------------------------
  describe("primary sale", () => {
    it("buy_tokens rejects a zero amount", async () => {
      await expectFail(buyTokens(investor, 0).rpc(), "InvalidPurchaseAmount");
    });

    it("buy_tokens rejects more than the vault holds", async () => {
      await expectFail(buyTokens(investor, TOKEN_SUPPLY + 1).rpc(), "InsufficientVaultSupply");
    });

    it("buy_tokens moves tokens to the investor and pays the creator", async () => {
      const before = await connection.getBalance(creator.publicKey);
      await buyTokens(investor, BUY_AMOUNT).rpc();
      const after = await connection.getBalance(creator.publicKey);

      expect(after - before).to.equal(BUY_AMOUNT * PRICE_PER_TOKEN);
      expect(await tokenBalance(ata(investor.publicKey, mint))).to.equal(BUY_AMOUNT);
      expect(await tokenBalance(vault)).to.equal(TOKEN_SUPPLY - BUY_AMOUNT);

      const pos = await acct.holderPosition.fetch(position(investor.publicKey));
      expect(pos.holder.toBase58()).to.equal(investor.publicKey.toBase58());
      expect(pos.totalClaimed.toNumber()).to.equal(0);
    });
  });

  // -------------------------------------------------------------------------
  describe("revenue distribution", () => {
    it("claim_revenue rejects when nothing has been deposited", async () => {
      await expectFail(claimRevenue(investor).rpc(), "NothingToClaim");
    });

    it("deposit_revenue rejects a zero amount", async () => {
      await expectFail(depositRevenue(creator, 0).rpc(), "InvalidDepositAmount");
    });

    it("deposit_revenue rejects anyone but the creator", async () => {
      await expectFail(depositRevenue(investor, DEPOSIT).rpc(), "Unauthorized");
    });

    it("deposit_revenue funds the pool and updates accounting", async () => {
      await depositRevenue(creator, DEPOSIT).rpc();

      const pool = await acct.revenuePool.fetch(revenuePool);
      expect(pool.totalDeposited.toNumber()).to.equal(DEPOSIT);
      expect(pool.accumulatedRewardsPerToken.gt(new BN(0))).to.be.true;
    });

    it("claim_revenue pays the holder their pro-rata share", async () => {
      const before = await connection.getBalance(investor.publicKey);
      await claimRevenue(investor).rpc();
      const after = await connection.getBalance(investor.publicKey);

      // 1% of supply -> 1% of the deposit
      expect(after - before).to.equal(DEPOSIT / 100);
      const pos = await acct.holderPosition.fetch(position(investor.publicKey));
      expect(pos.totalClaimed.toNumber()).to.equal(DEPOSIT / 100);
    });

    it("claim_revenue cannot be repeated without a new deposit", async () => {
      await expectFail(claimRevenue(investor).rpc(), "NothingToClaim");
    });

    it("tokens bought after a deposit earn nothing from it", async () => {
      await buyTokens(lateInvestor, 500).rpc();
      await expectFail(claimRevenue(lateInvestor).rpc(), "NothingToClaim");
    });

    it("claim_revenue rejects a wallet that never held tokens", async () => {
      await expectFail(claimRevenue(outsider).rpc());
    });
  });

  // -------------------------------------------------------------------------
  describe("secondary market", () => {
    const NONCE = 0;
    const LIST_AMOUNT = 400;
    const LIST_PRICE = 5_000;

    it("list_resale rejects a zero amount", async () => {
      await expectFail(listResale(investor, 0, LIST_PRICE, NONCE).rpc(), "InvalidListingAmount");
    });

    it("list_resale rejects a zero price", async () => {
      await expectFail(listResale(investor, LIST_AMOUNT, 0, NONCE).rpc(), "InvalidListingPrice");
    });

    it("list_resale rejects more than the seller holds", async () => {
      await expectFail(
        listResale(investor, BUY_AMOUNT + 1, LIST_PRICE, NONCE).rpc(),
        "InsufficientListedBalance"
      );
    });

    it("list_resale rejects a mint that is not the track's mint", async () => {
      // A worthless mint the seller controls, listed under the real track.
      const fakeMint = await createMint(connection, payer, payer.publicKey, null, 0);
      const fakeAta = await createAssociatedTokenAccount(
        connection,
        payer,
        fakeMint,
        investor.publicKey
      );
      await mintTo(connection, payer, fakeMint, fakeAta, payer, 1_000_000);

      await expectFail(
        program.methods
          .listResale(new BN(1), new BN(LIST_PRICE), new BN(99))
          .accounts({
            seller: investor.publicKey,
            track,
            sellerTokenAccount: fakeAta,
            mint: fakeMint,
            listing: listingPda(investor.publicKey, 99),
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([investor])
          .rpc(),
        "InvalidMint"
      );
    });

    it("list_resale records the listing and delegates the tokens", async () => {
      await listResale(investor, LIST_AMOUNT, LIST_PRICE, NONCE).rpc();

      const l = await acct.listing.fetch(listingPda(investor.publicKey, NONCE));
      expect(l.seller.toBase58()).to.equal(investor.publicKey.toBase58());
      expect(l.amount.toNumber()).to.equal(LIST_AMOUNT);
      expect(l.pricePerToken.toNumber()).to.equal(LIST_PRICE);
      // tokens stay with the seller until sold
      expect(await tokenBalance(ata(investor.publicKey, mint))).to.equal(BUY_AMOUNT);
    });

    it("cancel_resale rejects someone who is not the seller", async () => {
      await expectFail(cancelResale(buyer, investor.publicKey, NONCE).rpc());
    });

    it("buy_resale moves tokens and pays the seller", async () => {
      const before = await connection.getBalance(investor.publicKey);
      await buyResale(buyer, investor.publicKey, NONCE).rpc();
      const after = await connection.getBalance(investor.publicKey);

      expect(await tokenBalance(ata(buyer.publicKey, mint))).to.equal(LIST_AMOUNT);
      expect(await tokenBalance(ata(investor.publicKey, mint))).to.equal(BUY_AMOUNT - LIST_AMOUNT);
      // sale price plus the closed listing's rent
      expect(after - before).to.be.at.least(LIST_AMOUNT * LIST_PRICE);

      const closed = await connection.getAccountInfo(listingPda(investor.publicKey, NONCE));
      expect(closed).to.be.null;
    });

    it("a closed listing cannot be bought again", async () => {
      await expectFail(buyResale(lateInvestor, investor.publicKey, NONCE).rpc());
    });

    it("cancel_resale closes the listing", async () => {
      await listResale(investor, 100, LIST_PRICE, 1).rpc();
      await cancelResale(investor, investor.publicKey, 1).rpc();

      expect(await connection.getAccountInfo(listingPda(investor.publicKey, 1))).to.be.null;
      await expectFail(buyResale(buyer, investor.publicKey, 1).rpc());
    });
  });

  // -------------------------------------------------------------------------
  describe("revenue after a resale", () => {
    it("splits new revenue by current balances and keeps past revenue with the seller", async () => {
      await depositRevenue(creator, DEPOSIT).rpc();

      // investor: 600 tokens -> 0.6% of the second deposit, on top of the 1% already claimed
      await claimRevenue(investor).rpc();
      const inv = await acct.holderPosition.fetch(position(investor.publicKey));
      expect(inv.totalClaimed.toNumber()).to.equal(DEPOSIT / 100 + (DEPOSIT * 6) / 1000);

      // buyer: 400 tokens, bought after the first deposit -> only 0.4% of the second
      await claimRevenue(buyer).rpc();
      const buy = await acct.holderPosition.fetch(position(buyer.publicKey));
      expect(buy.totalClaimed.toNumber()).to.equal((DEPOSIT * 4) / 1000);

      // late investor: 500 tokens -> 0.5% of the second
      await claimRevenue(lateInvestor).rpc();
      const late = await acct.holderPosition.fetch(position(lateInvestor.publicKey));
      expect(late.totalClaimed.toNumber()).to.equal((DEPOSIT * 5) / 1000);
    });
  });
});
