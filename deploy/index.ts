import * as anchor from "@project-serum/anchor";
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet";
import { IDL } from "../target/types/token_lock";
import { PublicKey, SystemProgram, Transaction, Connection, Commitment, clusterApiUrl } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount, transfer } from "@solana/spl-token";
import { assert } from "chai";

function wait(milliseconds) {
  return new Promise((resolve) => {
    setTimeout(resolve, milliseconds);
  });
}

const tokenRecipient = new PublicKey("A2VFFS1PEL1wKL5Hp3TWvgARpa1W2KtXYvc2pVq6KhNE");
const tokenMint = new PublicKey("BZemhHtvSGZFMHTNj1m3nFxVJDittTjYYPgyu2d5fM7o")

describe("token-lock", () => {
  // Use Mainnet-fork for testing
  const commitment: Commitment = "confirmed";
  const connection = new Connection(clusterApiUrl("devnet"), {
    commitment,
    // wsEndpoint: "wss://api.devnet.solana.com/",
  });
  const options = anchor.AnchorProvider.defaultOptions();
  const wallet = NodeWallet.local();
  const provider = new anchor.AnchorProvider(connection, wallet, options);

  anchor.setProvider(provider);

  // CAUTTION: if you are intended to use the program that is deployed by yourself,
  // please make sure that the programIDs are consistent
  const programId = new PublicKey("5YFYMJ6zQyNzWDxXyDtA2nfJrPXnQRUaWkVyuAYencLw");
  const program = new anchor.Program(IDL, programId, provider);

  // Determined Seeds
  const adminSeed = "admin";
  const stateSeed = "state";

  const adminKey = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode(stateSeed)), Buffer.from(anchor.utils.bytes.utf8.encode(adminSeed))],
    program.programId
  )[0];

  it("init admin address", async () => {
    await program.methods
      .initAdmin(new anchor.BN(1726638815),new anchor.BN(5000),new anchor.BN(1000000))
      .accounts({
        admin: wallet.publicKey.toString(),
        adminState: adminKey.toString(),
        tokenMint,
        tokenRecipient,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([wallet.payer])
      .rpc();

    await wait(500);
    const fetchedAdminState: any = await program.account.adminState.fetch(adminKey);
    console.log(fetchedAdminState)
  });
});
