/// <reference types="vite/client" />

interface ImportMetaEnv {
	readonly VITE_APERIO_REGISTRY_KIND?: string;
	readonly VITE_APERIO_REGISTRY_ADDRESS?: string;
	readonly VITE_APERIO_REPO_ORGANIZATION?: string;
	readonly VITE_APERIO_REPO_NAME?: string;
	readonly VITE_APERIO_BUNDLE_GATEWAY?: string;
	readonly VITE_WS_URL?: string;
	readonly VITE_ETH_RPC_URL?: string;
	readonly VITE_LOCAL_WS_URL?: string;
	readonly VITE_LOCAL_ETH_RPC_URL?: string;
	readonly VITE_REGISTRY_FROM_BLOCK?: string;
	readonly VITE_LOG_CHUNK_SIZE?: string;
}

interface ImportMeta {
	readonly env: ImportMetaEnv;
}
