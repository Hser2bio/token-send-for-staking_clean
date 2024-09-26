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

const tokenRecipient = new PublicKey("2PQdqwMoV6y2gU3u9ijVhTtf4t4XsBEM4JvwHbwjqhaQ");
const tokenMint = new PublicKey("ErNQeVLdwxNrPXNT1FChKHfebMebSDYmNqw2yTfP35En")

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

  const vault = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), tokenMint.toBuffer()],
    program.programId
  )[0];

  // it("init admin address", async () => {
  //   await program.methods
  //     .initAdmin(new anchor.BN(1726638815),new anchor.BN(3600 * 24 * 365),new anchor.BN(10000000000000))
  //     .accounts({
  //       admin: wallet.publicKey.toString(),
  //       adminState: adminKey.toString(),
  //       tokenMint,
  //       tokenRecipient,
  //       vault,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     })
  //     .signers([wallet.payer])
  //     .rpc();

  //   await wait(500);
  //   const fetchedAdminState: any = await program.account.adminState.fetch(adminKey);
  //   console.log({fetchedAdminState, vault})
  // });

  it("send token", async () => {
    await program.methods
      .sendToken()
      .accounts({
        user: wallet.publicKey.toString(),
        adminState: adminKey.toString(),
        tokenMint,
        tokenRecipient,
        vault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([wallet.payer])
      .rpc();

    await wait(500);
    const fetchedAdminState: any = await program.account.adminState.fetch(adminKey);
    console.log({fetchedAdminState, vault, adminKey})
  });
});
