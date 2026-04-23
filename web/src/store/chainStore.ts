import { create } from "zustand";
import { getStoredEthRpcUrl, getStoredWsUrl } from "../config/network";

interface ChainState {
	wsUrl: string;
	ethRpcUrl: string;
	setWsUrl: (url: string) => void;
	setEthRpcUrl: (url: string) => void;
}

export const useChainStore = create<ChainState>((set) => ({
	wsUrl: getStoredWsUrl(),
	ethRpcUrl: getStoredEthRpcUrl(),
	setWsUrl: (wsUrl) => {
		localStorage.setItem("ws-url", wsUrl);
		set({ wsUrl });
	},
	setEthRpcUrl: (ethRpcUrl) => {
		localStorage.setItem("eth-rpc-url", ethRpcUrl);
		set({ ethRpcUrl });
	},
}));
