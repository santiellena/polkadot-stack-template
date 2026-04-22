import { deployments } from "./deployments";

export const ZERO_ADDRESS = "0x0000000000000000000000000000000000000000";

const registryKind = import.meta.env.VITE_APERIO_REGISTRY_KIND || "pvm";
const defaultDeploymentAddress =
	registryKind === "evm" ? deployments.evm : deployments.pvm;

export const DEFAULT_REGISTRY_ADDRESS =
	import.meta.env.VITE_APERIO_REGISTRY_ADDRESS ||
	defaultDeploymentAddress ||
	null;

export const DEFAULT_REPO_ORGANIZATION = import.meta.env.VITE_APERIO_REPO_ORGANIZATION || "";
export const DEFAULT_REPO_NAME = import.meta.env.VITE_APERIO_REPO_NAME || "";

export const BUNDLE_GATEWAY_BASE =
	(import.meta.env.VITE_APERIO_BUNDLE_GATEWAY || "https://paseo-ipfs.polkadot.io/ipfs").replace(
		/\/$/,
		"",
	);
