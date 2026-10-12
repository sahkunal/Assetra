/**
 * One-time devnet setup: creates the ProgramConfig account.
 *
 *   admin    = your CLI wallet (~/.config/solana/id.json)
 *   verifier = keys/verifier.json (generated here if missing, git-ignored)
 *
 * Safe to re-run: if the config already exists it just prints it.
 *
 *   npx ts-node --transpile-only scripts/init-config.ts
 */
import * as anchor from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";

const RPC_URL = process.env.RPC_URL || "https://api.devnet.solana.com";
const WALLET_PATH =
  process.env.ANCHOR_WALLET ?? path.join(os.homedir(), ".config/solana/id.json");
const VERIFIER_PATH = path.join(__dirname, "..", "keys", "verifier.json");
const VERIFIER_FUNDING = 0.5 * LAMPORTS_PER_SOL;

function loadKeypair(file: string): Keypair {
  return Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(file, "utf8"))));
}

function loadOrCreateVerifier(): Keypair {
  if (fs.existsSync(VERIFIER_PATH)) return loadKeypair(VERIFIER_PATH);
  fs.mkdirSync(path.dirname(VERIFIER_PATH), { recursive: true });
  const kp = Keypair.generate();
  fs.writeFileSync(VERIFIER_PATH, JSON.stringify(Array.from(kp.secretKey)), { mode: 0o600 });
  console.log("Generated new verifier keypair ->", VERIFIER_PATH);
  return kp;
}

async function main() {
  const admin = loadKeypair(WALLET_PATH);
  const connection = new anchor.web3.Connection(RPC_URL, "confirmed");
  const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(admin), {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);

  const idl = JSON.parse(
    fs.readFileSync(path.join(__dirname, "..", "idls", "assetra.json"), "utf8")
  );
  const program = new anchor.Program(idl, provider);

  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    program.programId
  );

  console.log("Program :", program.programId.toBase58());
  console.log("Admin   :", admin.publicKey.toBase58());
  console.log("Config  :", configPda.toBase58());

  const existing = await connection.getAccountInfo(configPda);
  if (existing) {
    const cfg: any = await (program.account as any).programConfig.fetch(configPda);
    console.log("\nConfig already initialized:");
    console.log("  admin   :", cfg.admin.toBase58());
    console.log("  verifier:", cfg.verifier.toBase58());
    return;
  }

  const verifier = loadOrCreateVerifier();
  console.log("Verifier:", verifier.publicKey.toBase58());

  const sig = await program.methods
    .initializeConfig(verifier.publicKey)
    .accounts({ admin: admin.publicKey })
    .rpc();
  console.log("\ninitialize_config tx:", sig);

  // The verifier signs verify_track later, so it needs SOL for fees.
  const balance = await connection.getBalance(verifier.publicKey);
  if (balance < VERIFIER_FUNDING / 2) {
    const tx = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: admin.publicKey,
        toPubkey: verifier.publicKey,
        lamports: VERIFIER_FUNDING,
      })
    );
    const fundSig = await provider.sendAndConfirm(tx);
    console.log("Funded verifier with 0.5 SOL:", fundSig);
  }

  console.log("\nDone. Back up keys/verifier.json — it controls track verification.");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
