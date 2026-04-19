#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { createNanoEvents } from "nanoevents";
import { fromAsyncThrowable } from "neverthrow";

const DEFAULT_TIMEOUT_MS = 300_000;
const DEFAULT_ALLOWANCE_WAIT_MS = 30_000;
const DEFAULT_ENDPOINT = "wss://pop3-testnet.parity-lab.parity.io/people";
const DEFAULT_APP_ID = "crrp-cli";
const DEFAULT_METADATA =
  "https://gist.githubusercontent.com/valentunn/97938ca74b8d984f62ec95c7e633e24f/raw/b52f8ca43d8c3661d4360b16ca54652ad0a4f664/test_metadata.json";

let depsPromise = null;

async function loadDependencies() {
  if (depsPromise) {
    return depsPromise;
  }

  depsPromise = (async () => {
    try {
      const [{ createPappAdapter }, statementStore, wsProvider, qrModule] =
        await Promise.all([
          import("@novasamatech/host-papp"),
          import("@novasamatech/statement-store"),
          import("polkadot-api/ws-provider"),
          import("qrcode-terminal"),
        ]);

      const qrcodeTerminal = qrModule.default ?? qrModule;
      return {
        createPappAdapter,
        createLazyClient: statementStore.createLazyClient,
        createPapiStatementStoreAdapter: statementStore.createPapiStatementStoreAdapter,
        getWsProvider: wsProvider.getWsProvider,
        qrcodeTerminal,
      };
    } catch (error) {
      if (
        error &&
        error.code === "ERR_UNKNOWN_FILE_EXTENSION" &&
        typeof error.message === "string" &&
        error.message.includes(".wasm")
      ) {
        throw new Error(
          "Node.js needs WASM ESM support for host-papp. Re-run with: node --experimental-wasm-modules ...",
        );
      }

      throw error;
    }
  })();

  return depsPromise;
}

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

  const withoutPrefix =
    trimmed.startsWith("0x") || trimmed.startsWith("0X") ? trimmed.slice(2) : trimmed;
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

function storageKeyForStatementAllowance(accountIdBytes) {
  const prefix = Buffer.from(":statement-allowance:", "utf8");
  const keyBytes = Buffer.concat([prefix, Buffer.from(accountIdBytes)]);
  return `0x${keyBytes.toString("hex")}`;
}

async function readJsonObjectFile(filePath) {
  try {
    const content = await fs.readFile(filePath, "utf8");
    const parsed = JSON.parse(content);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return {};
    }
    return parsed;
  } catch {
    return {};
  }
}

function createFileStorageAdapter(storeFilePath, initialState) {
  const events = createNanoEvents();
  const storage = { ...initialState };

  const persist = async () => {
    const directory = path.dirname(storeFilePath);
    await fs.mkdir(directory, { recursive: true });
    await fs.writeFile(storeFilePath, `${JSON.stringify(storage, null, 2)}\n`, "utf8");
  };

  return {
    write: fromAsyncThrowable(async (key, value) => {
      storage[key] = value;
      await persist();
      events.emit(key, value);
    }),
    read: fromAsyncThrowable(async (key) => storage[key] ?? null),
    clear: fromAsyncThrowable(async (key) => {
      delete storage[key];
      await persist();
      events.emit(key, null);
    }),
    subscribe(key, callback) {
      return events.on(key, callback);
    },
  };
}

function resolveStorageFilePath(args, sessionOutPath) {
  const explicit = args["storage-file"]?.trim();
  if (explicit) {
    return explicit;
  }

  if (sessionOutPath) {
    return path.join(path.dirname(sessionOutPath), "hostpapp-storage.json");
  }

  return ".crrp/hostpapp-storage.json";
}

async function assertStatementAllowance(
  lazyClient,
  localAccountIdBytes,
  endpoint,
  maxWaitMs = DEFAULT_ALLOWANCE_WAIT_MS,
) {
  const requestFn = lazyClient.getRequestFn();
  const allowanceKey = storageKeyForStatementAllowance(localAccountIdBytes);
  const methods = ["chain_getStorage", "state_getStorage"];
  let lastErrors = [];
  const start = Date.now();

  // Poll briefly to reduce first-run false negatives while allowance converges.
  while (true) {
    const errors = [];
    for (const method of methods) {
      try {
        const value = await requestFn(method, [allowanceKey]);
        if (value !== null && value !== "0x") {
          return;
        }
      } catch (error) {
        errors.push(`${method}: ${error instanceof Error ? error.message : String(error)}`);
      }
    }
    lastErrors = errors;

    const elapsed = Date.now() - start;
    if (elapsed >= maxWaitMs) {
      break;
    }

    process.stderr.write(
      `[hostpapp] waiting for statement allowance (${Math.round((maxWaitMs - elapsed) / 1000)}s left)\n`,
    );
    await sleep(3000);
  }

  const localKeyHex = bytesToHex(localAccountIdBytes);
  const detail =
    lastErrors.length > 0
      ? ` RPC errors: ${lastErrors.join(" | ")}.`
      : " Storage key was empty or missing.";

  throw new Error(
    `No statement-store allowance for host session key ${localKeyHex} on ${endpoint}. ` +
      `Signing requests cannot be submitted, so pwallet cannot receive approval modals. ` +
      `Provision allowance for this key (storage key ${allowanceKey}) and retry.${detail}`,
  );
}

function parseTimeout(raw) {
  if (raw == null) {
    return DEFAULT_TIMEOUT_MS;
  }

  const parsed = Number(raw);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`Invalid timeout value: ${raw}`);
  }

  return parsed;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
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

function addressFromAccountId(accountId) {
  return bytesToHex(accountId);
}

function writeQr(qrcodeTerminal, payload) {
  process.stderr.write("Scan this QR with pwallet:\n");
  qrcodeTerminal.generate(payload, { small: true }, (qr) => {
    process.stderr.write(`${qr}\n`);
  });
  process.stderr.write(`Deep link: ${payload}\n`);
}

async function createBridgeContext(args) {
  const {
    createPappAdapter,
    createLazyClient,
    createPapiStatementStoreAdapter,
    getWsProvider,
    qrcodeTerminal,
  } = await loadDependencies();

  const endpoint =
    (typeof args.endpoint === "string" && args.endpoint.trim()) ||
    process.env.CRRP_PAPP_TERM_ENDPOINT ||
    DEFAULT_ENDPOINT;
  const appId =
    (typeof args["app-id"] === "string" && args["app-id"].trim()) ||
    process.env.CRRP_PAPP_APP_ID ||
    DEFAULT_APP_ID;
  const metadata =
    (typeof args.metadata === "string" && args.metadata.trim()) ||
    process.env.CRRP_PAPP_METADATA ||
    DEFAULT_METADATA;
  const timeoutMs = parseTimeout(args["timeout-ms"]);
  const allowanceWaitMs = parseTimeout(args["allowance-wait-ms"] ?? DEFAULT_ALLOWANCE_WAIT_MS);
  const sessionOut = args["session-out"]?.trim();
  const storageFile = resolveStorageFilePath(args, sessionOut);
  const storageState = await readJsonObjectFile(storageFile);

  process.stderr.write(`[hostpapp] endpoint=${endpoint}\n`);
  process.stderr.write(`[hostpapp] appId=${appId}\n`);
  process.stderr.write(`[hostpapp] storage=${storageFile}\n`);

  const lazyClient = createLazyClient(
    getWsProvider([endpoint], {
      heartbeatTimeout: 120_000,
    }),
  );
  const statementStore = createPapiStatementStoreAdapter(lazyClient);
  const storage = createFileStorageAdapter(storageFile, storageState);
  const adapter = createPappAdapter({
    appId,
    metadata,
    adapters: {
      lazyClient,
      statementStore,
      storage,
    },
  });

  return {
    adapter,
    lazyClient,
    qrcodeTerminal,
    endpoint,
    appId,
    metadata,
    timeoutMs,
    allowanceWaitMs,
    sessionOut,
    storageFile,
  };
}

async function ensureAuthenticatedSession(adapter, qrcodeTerminal, timeoutMs) {
  let qrShown = false;
  const stopPairingSubscription = adapter.sso.pairingStatus.subscribe((status) => {
    if (status.step === "pairing" && !qrShown) {
      qrShown = true;
      writeQr(qrcodeTerminal, status.payload);
    }

    if (status.step === "pairingError") {
      process.stderr.write(`[hostpapp] pairing error: ${status.message}\n`);
    }

    if (status.step === "finished") {
      process.stderr.write(`[hostpapp] pairing finished (session ${status.session.id})\n`);
    }
  });

  const stopAttestationSubscription = adapter.sso.attestationStatus.subscribe((status) => {
    if (status.step === "attestation") {
      process.stderr.write(`[hostpapp] attestation in progress (username ${status.username})\n`);
    }

    if (status.step === "attestationError") {
      process.stderr.write(`[hostpapp] attestation error: ${status.message}\n`);
    }

    if (status.step === "finished") {
      process.stderr.write("[hostpapp] attestation finished\n");
    }
  });

  let sessions = adapter.sessions.sessions.read();
  let session = Array.isArray(sessions) && sessions.length > 0 ? sessions[0] : null;

  if (session) {
    process.stderr.write(`[hostpapp] reusing stored session ${session.id}\n`);
    stopPairingSubscription();
    stopAttestationSubscription();
    return session;
  }

  let authResult;
  try {
    authResult = await withTimeout(adapter.sso.authenticate(), timeoutMs, "host-papp authenticate");
  } finally {
    stopPairingSubscription();
    stopAttestationSubscription();
  }

  if (authResult.isErr()) {
    throw new Error(`host-papp authentication failed: ${authResult.error.message}`);
  }

  if (authResult.value == null) {
    throw new Error("host-papp authentication was aborted before creating a session");
  }

  sessions = adapter.sessions.sessions.read();
  if (!Array.isArray(sessions) || sessions.length === 0) {
    throw new Error("host-papp returned no active sessions after authentication");
  }

  session = sessions.find((entry) => entry.id === authResult.value.id) ?? sessions[0];
  if (!session) {
    throw new Error("host-papp session is unavailable");
  }

  return session;
}

async function readSessionSecrets(adapter, session) {
  const secretResult = await adapter.secrets.read(session.id);
  if (secretResult.isErr()) {
    throw new Error(`failed reading host-papp session secrets: ${secretResult.error.message}`);
  }

  if (secretResult.value == null) {
    throw new Error("host-papp returned null secret payload for authenticated session");
  }

  return secretResult.value;
}

function buildSessionOutput(session, secrets, endpoint, appId, metadata) {
  const address = addressFromAccountId(session.remoteAccount.accountId);
  return {
    endpoint,
    app_id: appId,
    metadata,
    session_id: session.id,
    address,
    local_account_id_hex: bytesToHex(session.localAccount.accountId),
    remote_account_id_hex: bytesToHex(session.remoteAccount.accountId),
    remote_public_key_hex: bytesToHex(session.remoteAccount.publicKey),
    local_secret_hex: bytesToHex(secrets.ssSecret),
    local_entropy_hex: bytesToHex(secrets.entropy),
    local_encr_secret_hex: bytesToHex(secrets.encrSecret),
  };
}

async function persistSessionIfRequested(sessionOut, storageFile, output) {
  if (!sessionOut) {
    return;
  }

  const now = Math.floor(Date.now() / 1000);
  const persistedSession = {
    backend: "papp-hostpapp",
    session_id: output.session_id,
    created_at_unix_secs: now,
    wallet_label: "pwallet",
    chain: output.endpoint,
    accounts: [output.address],
    local_account_id_hex: output.local_account_id_hex,
    remote_account_id_hex: output.remote_account_id_hex,
    remote_public_key_hex: output.remote_public_key_hex,
    local_secret_hex: output.local_secret_hex,
    local_entropy_hex: output.local_entropy_hex,
    local_encr_secret_hex: output.local_encr_secret_hex,
    metadata_url: output.metadata,
    app_id: output.app_id,
    storage_file: storageFile,
  };

  await fs.mkdir(path.dirname(sessionOut), { recursive: true });
  await fs.writeFile(sessionOut, `${JSON.stringify(persistedSession, null, 2)}\n`, "utf8");
  process.stderr.write(`[hostpapp] wrote session export to ${sessionOut}\n`);
}

async function authWithHostPapp(args) {
  const context = await createBridgeContext(args);
  const session = await ensureAuthenticatedSession(
    context.adapter,
    context.qrcodeTerminal,
    context.timeoutMs,
  );
  const secrets = await readSessionSecrets(context.adapter, session);

  const output = buildSessionOutput(
    session,
    secrets,
    context.endpoint,
    context.appId,
    context.metadata,
  );
  await persistSessionIfRequested(context.sessionOut, context.storageFile, output);
  process.stdout.write(`${JSON.stringify(output)}\n`);
  process.exit(0);
}

async function signRawWithHostPapp(args) {
  const payloadHex = requiredArg(args, "payload-hex");
  const payloadBytes = hexToBytes(payloadHex);

  const context = await createBridgeContext(args);
  const session = await ensureAuthenticatedSession(
    context.adapter,
    context.qrcodeTerminal,
    context.timeoutMs,
  );

  await assertStatementAllowance(
    context.lazyClient,
    session.localAccount.accountId,
    context.endpoint,
    context.allowanceWaitMs,
  );
  process.stderr.write("[hostpapp] statement allowance present for local session key\n");

  const address = addressFromAccountId(session.remoteAccount.accountId);

  process.stderr.write(`[hostpapp] requesting signRaw from wallet for session ${session.id}\n`);
  const signResult = await withTimeout(
    session.signRaw({
      address,
      data: {
        tag: "Bytes",
        value: payloadBytes,
      },
    }),
    context.timeoutMs,
    "host-papp signRaw",
  );

  if (signResult.isErr()) {
    throw new Error(`wallet rejected signRaw request: ${signResult.error.message}`);
  }

  const secrets = await readSessionSecrets(context.adapter, session);
  const baseOutput = buildSessionOutput(
    session,
    secrets,
    context.endpoint,
    context.appId,
    context.metadata,
  );

  const now = Math.floor(Date.now() / 1000);
  const output = {
    ...baseOutput,
    request_id: `${session.id}:${now}`,
    signature_hex: bytesToHex(signResult.value.signature),
    signed_transaction_hex: signResult.value.signedTransaction
      ? bytesToHex(signResult.value.signedTransaction)
      : null,
  };

  await persistSessionIfRequested(context.sessionOut, context.storageFile, baseOutput);
  process.stdout.write(`${JSON.stringify(output)}\n`);
  process.exit(0);
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const command = args._[0];

  if (command === "auth") {
    await authWithHostPapp(args);
    return;
  }

  if (command === "sign-raw") {
    await signRawWithHostPapp(args);
    return;
  }

  throw new Error("Unsupported command. Use: auth or sign-raw");
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
