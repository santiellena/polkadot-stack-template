#!/usr/bin/env node

import fs from "node:fs/promises";
import process from "node:process";

const DEFAULT_TIMEOUT_MS = 300_000;
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
      const [{ createPappAdapter }, statementStore, storageAdapter, wsProvider, qrModule] =
        await Promise.all([
          import("@novasamatech/host-papp"),
          import("@novasamatech/statement-store"),
          import("@novasamatech/storage-adapter"),
          import("polkadot-api/ws-provider"),
          import("qrcode-terminal"),
        ]);

      const qrcodeTerminal = qrModule.default ?? qrModule;
      return {
        createPappAdapter,
        createLazyClient: statementStore.createLazyClient,
        createPapiStatementStoreAdapter: statementStore.createPapiStatementStoreAdapter,
        createMemoryAdapter: storageAdapter.createMemoryAdapter,
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

async function signRawWithHostPapp(args) {
  const {
    createPappAdapter,
    createLazyClient,
    createPapiStatementStoreAdapter,
    createMemoryAdapter,
    getWsProvider,
    qrcodeTerminal,
  } = await loadDependencies();

  const payloadHex = requiredArg(args, "payload-hex");
  const payloadBytes = hexToBytes(payloadHex);

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

  process.stderr.write(`[hostpapp] endpoint=${endpoint}\n`);
  process.stderr.write(`[hostpapp] appId=${appId}\n`);

  const lazyClient = createLazyClient(
    getWsProvider([endpoint], {
      heartbeatTimeout: 120_000,
    }),
  );
  const statementStore = createPapiStatementStoreAdapter(lazyClient);
  const storage = createMemoryAdapter();
  const adapter = createPappAdapter({
    appId,
    metadata,
    adapters: {
      lazyClient,
      statementStore,
      storage,
    },
  });

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

  let authResult;
  try {
    authResult = await withTimeout(
      adapter.sso.authenticate(),
      timeoutMs,
      "host-papp authenticate",
    );
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

  const sessions = adapter.sessions.sessions.read();
  if (!Array.isArray(sessions) || sessions.length === 0) {
    throw new Error("host-papp returned no active sessions after authentication");
  }

  const session = sessions.find((entry) => entry.id === authResult.value.id) ?? sessions[0];
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
    timeoutMs,
    "host-papp signRaw",
  );

  if (signResult.isErr()) {
    throw new Error(`wallet rejected signRaw request: ${signResult.error.message}`);
  }

  const secretResult = await adapter.secrets.read(session.id);
  if (secretResult.isErr()) {
    throw new Error(`failed reading host-papp session secrets: ${secretResult.error.message}`);
  }

  if (secretResult.value == null) {
    throw new Error("host-papp returned null secret payload for authenticated session");
  }

  const now = Math.floor(Date.now() / 1000);
  const output = {
    request_id: `${session.id}:${now}`,
    endpoint,
    app_id: appId,
    metadata,
    session_id: session.id,
    address,
    local_account_id_hex: bytesToHex(session.localAccount.accountId),
    remote_account_id_hex: bytesToHex(session.remoteAccount.accountId),
    remote_public_key_hex: bytesToHex(session.remoteAccount.publicKey),
    local_secret_hex: bytesToHex(secretResult.value.ssSecret),
    local_entropy_hex: bytesToHex(secretResult.value.entropy),
    local_encr_secret_hex: bytesToHex(secretResult.value.encrSecret),
    signature_hex: bytesToHex(signResult.value.signature),
    signed_transaction_hex: signResult.value.signedTransaction
      ? bytesToHex(signResult.value.signedTransaction)
      : null,
  };

  const sessionOut = args["session-out"]?.trim();
  if (sessionOut) {
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
    };
    await fs.writeFile(sessionOut, `${JSON.stringify(persistedSession, null, 2)}\n`, "utf8");
    process.stderr.write(`[hostpapp] wrote session export to ${sessionOut}\n`);
  }

  process.stdout.write(`${JSON.stringify(output)}\n`);
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const command = args._[0];

  if (command === "sign-raw") {
    await signRawWithHostPapp(args);
    return;
  }

  throw new Error("Unsupported command. Use: sign-raw");
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
