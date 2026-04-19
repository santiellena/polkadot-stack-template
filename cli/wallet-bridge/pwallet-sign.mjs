#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs/promises";
import process from "node:process";

import {
  createEncryption,
  createLazyClient,
  createLocalSessionAccount,
  createPapiStatementStoreAdapter,
  createRemoteSessionAccount,
  createSession,
  createSr25519Derivation,
  createSr25519Prover,
  deriveSr25519PublicKey,
} from "@novasamatech/statement-store";
import { entropyToMnemonic } from "@polkadot-labs/hdkd-helpers";
import { secretFromSeed as sr25519SecretFromSeed } from "@scure/sr25519";
import { Bytes, Enum, Option, Struct, Vector, str, u32, _void } from "scale-ts";
import { Result } from "scale-ts";
import { Hex, OptionBool } from "@novasamatech/scale";
import { getWsProvider } from "polkadot-api/ws-provider";

const DEFAULT_TIMEOUT_MS = 180_000;
const DEFAULT_ACK_TIMEOUT_MS = 12_000;
const DEFAULT_ENDPOINT = "wss://pop3-testnet.parity-lab.parity.io/people";

const SignPayloadRequestCodec = Struct({
  address: str,
  blockHash: Hex(),
  blockNumber: Hex(),
  era: Hex(),
  genesisHash: Hex(),
  method: Hex(),
  nonce: Hex(),
  specVersion: Hex(),
  tip: Hex(),
  transactionVersion: Hex(),
  signedExtensions: Vector(str),
  version: u32,
  assetId: Option(Hex()),
  metadataHash: Option(Hex()),
  mode: Option(u32),
  withSignedTransaction: OptionBool,
});

const SignRawRequestCodec = Struct({
  address: str,
  data: Enum({
    Bytes: Bytes(),
    Payload: str,
  }),
});

const SigningRequestCodec = Enum({
  Payload: SignPayloadRequestCodec,
  Raw: SignRawRequestCodec,
});

const SignPayloadResponseDataCodec = Struct({
  signature: Bytes(),
  signedTransaction: Option(Bytes()),
});

const SignPayloadResponseCodec = Struct({
  respondingTo: str,
  payload: Result(SignPayloadResponseDataCodec, str),
});

const RemoteMessageCodec = Struct({
  messageId: str,
  data: Enum({
    v1: Enum({
      Disconnected: _void,
      SignRequest: SigningRequestCodec,
      SignResponse: SignPayloadResponseCodec,
      RingVrfAliasRequest: Bytes(),
      RingVrfAliasResponse: Bytes(),
    }),
  }),
});

function parseArgs(argv) {
  const args = { _: [] };
  for (let i = 0; i < argv.length; i++) {
    const token = argv[i];
    if (!token.startsWith("--")) {
      args._.push(token);
      continue;
    }

    const key = token.slice(2);
    const value = argv[i + 1];
    if (!value || value.startsWith("--")) {
      throw new Error(`Missing value for --${key}`);
    }
    args[key] = value;
    i++;
  }
  return args;
}

function requiredArg(args, key) {
  const value = args[key];
  if (!value || typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`Missing required argument --${key}`);
  }
  return value.trim();
}

function normalizeHex(value) {
  const trimmed = String(value ?? "").trim();
  if (trimmed.length === 0) {
    throw new Error("Hex value cannot be empty");
  }
  const withoutPrefix = trimmed.startsWith("0x") || trimmed.startsWith("0X")
    ? trimmed.slice(2)
    : trimmed;
  if (withoutPrefix.length === 0) {
    throw new Error("Hex value cannot be empty");
  }
  if (withoutPrefix.length % 2 !== 0) {
    throw new Error(`Invalid hex length ${withoutPrefix.length}; expected even length`);
  }
  if (!/^[0-9a-fA-F]+$/.test(withoutPrefix)) {
    throw new Error("Hex value contains non-hex characters");
  }
  return `0x${withoutPrefix.toLowerCase()}`;
}

function hexToBytes(value) {
  const normalized = normalizeHex(value);
  return Uint8Array.from(Buffer.from(normalized.slice(2), "hex"));
}

function bytesToHex(bytes) {
  if (!(bytes instanceof Uint8Array)) {
    throw new Error("Expected Uint8Array");
  }
  return `0x${Buffer.from(bytes).toString("hex")}`;
}

function derivePappLocalSecretFromEntropy(entropy, derivationPath = "//wallet//sso") {
  const mnemonic = entropyToMnemonic(entropy).normalize("NFKD");
  const pbkdf2Seed = crypto.pbkdf2Sync(mnemonic, "mnemonic", 2048, 64, "sha512");
  const miniSecret = Uint8Array.from(pbkdf2Seed.slice(0, 32));
  const baseSecret = sr25519SecretFromSeed(miniSecret);
  return createSr25519Derivation(baseSecret, derivationPath);
}

function ensureByteArray(value, label) {
  if (value instanceof Uint8Array) {
    return value;
  }
  if (Array.isArray(value)) {
    return Uint8Array.from(value);
  }
  throw new Error(`Expected ${label} to be bytes`);
}

function parsePayloadResult(resultLike) {
  if (!resultLike || typeof resultLike !== "object") {
    return { ok: false, error: "Invalid response payload" };
  }

  if (typeof resultLike.success === "boolean") {
    if (resultLike.success) {
      return { ok: true, value: resultLike.value };
    }
    return { ok: false, error: String(resultLike.value ?? "Wallet rejected request") };
  }

  if (typeof resultLike.tag === "string") {
    if (resultLike.tag === "Ok") {
      return { ok: true, value: resultLike.value };
    }
    if (resultLike.tag === "Err") {
      return { ok: false, error: String(resultLike.value ?? "Wallet rejected request") };
    }
  }

  if (resultLike.signature) {
    return { ok: true, value: resultLike };
  }

  return { ok: false, error: "Unsupported SignResponse payload shape" };
}

function unwrapRemotePayload(message) {
  if (!message || message.type !== "request") {
    return null;
  }

  const payload = message.payload;
  if (!payload) {
    return null;
  }

  if (payload.status === "failed") {
    return null;
  }

  return payload.status === "parsed" ? payload.value : payload;
}

async function createSigningSession(sessionData, endpoint) {
  const localAccountIdHex = sessionData.local_account_id_hex;
  const remoteAccountIdHex = sessionData.remote_account_id_hex;
  const sharedSecretHex = sessionData.shared_secret_hex;
  const localEntropyHex = sessionData.local_entropy_hex;

  if (!localAccountIdHex || !remoteAccountIdHex || !sharedSecretHex || !localEntropyHex) {
    throw new Error(
      "wallet-session.json is missing required pwallet signing fields (including local_entropy_hex). Re-run wallet sign-in.",
    );
  }

  const localAccountId = hexToBytes(localAccountIdHex);
  const remoteAccountId = hexToBytes(remoteAccountIdHex);
  const sharedSecret = hexToBytes(sharedSecretHex);
  const localSecret = derivePappLocalSecretFromEntropy(hexToBytes(localEntropyHex), "//wallet//sso");

  const localPub = bytesToHex(deriveSr25519PublicKey(localSecret));
  if (localPub.toLowerCase() !== normalizeHex(localAccountIdHex).toLowerCase()) {
    throw new Error(
      "wallet-session.json has inconsistent local signing material (derived pubkey does not match local_account_id_hex). Re-run wallet sign-in.",
    );
  }

  const statementStore = createPapiStatementStoreAdapter(
    createLazyClient(
      getWsProvider([endpoint], {
        heartbeatTimeout: 120_000,
      }),
    ),
  );

  const localAccount = createLocalSessionAccount(localAccountId);
  const remoteAccount = createRemoteSessionAccount(remoteAccountId, sharedSecret);
  const encryption = createEncryption(sharedSecret);
  const prover = createSr25519Prover(localSecret);

  return createSession({
    localAccount,
    remoteAccount,
    statementStore,
    encryption,
    prover,
  });
}

function waitForSignResponse(session, requestMessageId, timeoutMs) {
  return new Promise((resolve, reject) => {
    let unsubscribe = null;
    const timer = setTimeout(() => {
      if (unsubscribe) {
        try {
          unsubscribe();
        } catch {
          // noop
        }
      }
      reject(
        new Error(
          `Timed out waiting for pwallet signature response after ${timeoutMs}ms (transport ack may have succeeded, but no SignResponse was received). Open pwallet, keep it in foreground, and approve the pending signing request.`,
        ),
      );
    }, timeoutMs);

    unsubscribe = session.subscribe(RemoteMessageCodec, (messages) => {
      for (const message of messages) {
        const remoteMessage = unwrapRemotePayload(message);
        if (!remoteMessage?.data || remoteMessage.data.tag !== "v1") {
          continue;
        }

        const v1 = remoteMessage.data.value;
        if (!v1 || v1.tag !== "SignResponse") {
          continue;
        }

        const signResponse = v1.value;
        if (!signResponse || signResponse.respondingTo !== requestMessageId) {
          continue;
        }

        const parsed = parsePayloadResult(signResponse.payload);
        if (!parsed.ok) {
          clearTimeout(timer);
          if (unsubscribe) {
            try {
              unsubscribe();
            } catch {
              // noop
            }
          }
          reject(new Error(parsed.error || "Wallet rejected signature request"));
          return;
        }

        const value = parsed.value ?? {};
        const signature = ensureByteArray(value.signature, "signature");
        const signedTransaction =
          value.signedTransaction == null
            ? null
            : ensureByteArray(value.signedTransaction, "signedTransaction");

        clearTimeout(timer);
        if (unsubscribe) {
          try {
            unsubscribe();
          } catch {
            // noop
          }
        }

        resolve({
          responseMessageId: remoteMessage.messageId,
          signature,
          signedTransaction,
        });
        return;
      }
    });
  });
}

function withTimeout(promise, timeoutMs, label) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(`${label} timed out after ${timeoutMs}ms`));
    }, timeoutMs);

    promise.then(
      (value) => {
        clearTimeout(timer);
        resolve(value);
      },
      (error) => {
        clearTimeout(timer);
        reject(error);
      },
    );
  });
}

async function signRaw(args) {
  const sessionFile = requiredArg(args, "session-file");
  const payloadHex = requiredArg(args, "payload-hex");
  const payloadBytes = hexToBytes(payloadHex);

  const sessionData = JSON.parse(await fs.readFile(sessionFile, "utf8"));
  const endpoint =
    (typeof args.endpoint === "string" && args.endpoint.trim()) ||
    (typeof sessionData.chain === "string" && sessionData.chain.trim()) ||
    DEFAULT_ENDPOINT;

  const address =
    (typeof args.address === "string" && args.address.trim()) ||
    (Array.isArray(sessionData.accounts) && typeof sessionData.accounts[0] === "string"
      ? sessionData.accounts[0]
      : sessionData.remote_account_id_hex ||
        sessionData.local_account_id_hex ||
        "0x");

  const timeoutMs = args["timeout-ms"] ? Number(args["timeout-ms"]) : DEFAULT_TIMEOUT_MS;
  if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) {
    throw new Error(`Invalid --timeout-ms value: ${args["timeout-ms"]}`);
  }
  const ackTimeoutMs = args["ack-timeout-ms"] ? Number(args["ack-timeout-ms"]) : DEFAULT_ACK_TIMEOUT_MS;
  if (!Number.isFinite(ackTimeoutMs) || ackTimeoutMs <= 0) {
    throw new Error(`Invalid --ack-timeout-ms value: ${args["ack-timeout-ms"]}`);
  }

  console.error(`[bridge] endpoint=${endpoint}`);
  console.error(`[bridge] loading session from ${sessionFile}`);
  const session = await createSigningSession(sessionData, endpoint);
  console.error("[bridge] session created");

  const requestMessageId = crypto.randomUUID();
  const signResponsePromise = waitForSignResponse(session, requestMessageId, timeoutMs);

  const signRequest = {
    messageId: requestMessageId,
    data: {
      tag: "v1",
      value: {
        tag: "SignRequest",
        value: {
          tag: "Raw",
          value: {
            address,
            data: {
              tag: "Bytes",
              value: payloadBytes,
            },
          },
        },
      },
    },
  };

  console.error(`[bridge] submitting SignRequest messageId=${requestMessageId}`);
  const submitResult = await session.submitRequestMessage(RemoteMessageCodec, signRequest);
  if (submitResult?.isErr?.()) {
    throw new Error(`SignRequest submission failed: ${String(submitResult.error)}`);
  }

  const transportToken = submitResult?.value?.requestId;
  if (!transportToken) {
    throw new Error("SignRequest submission returned no transport request token");
  }
  console.error(`[bridge] transport token=${transportToken}`);

  console.error(
    `[bridge] waiting for transport ACK from wallet (up to ${ackTimeoutMs}ms, non-blocking)...`,
  );
  const ackPromise = withTimeout(
    session.waitForResponseMessage(transportToken),
    ackTimeoutMs,
    "Wallet transport ACK",
  )
    .then(() => {
      console.error("[bridge] wallet transport ACK received");
      return true;
    })
    .catch((error) => {
      console.error(
        `[bridge] wallet transport ACK missing: ${error instanceof Error ? error.message : String(error)}`,
      );
      return false;
    });

  console.error("[bridge] waiting for SignResponse (user approval)...");
  console.error("[bridge] check pwallet app for a pending signing request modal.");
  const response = await signResponsePromise;
  await ackPromise;
  if (typeof session.dispose === "function") {
    session.dispose();
  }

  const output = {
    request_id: requestMessageId,
    response_message_id: response.responseMessageId,
    endpoint,
    address,
    signature_hex: bytesToHex(response.signature),
    signed_transaction_hex: response.signedTransaction ? bytesToHex(response.signedTransaction) : null,
  };

  process.stdout.write(`${JSON.stringify(output)}\n`);
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const command = args._[0];

  if (command === "sign-raw") {
    await signRaw(args);
    return;
  }

  throw new Error("Unsupported command. Use: sign-raw");
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
