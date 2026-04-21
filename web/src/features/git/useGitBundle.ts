import { useEffect, useState } from "react";
import * as git from "isomorphic-git";
import { parseBundleHeader } from "./parseBundleHeader";
import { createMemFs } from "./memfs";

export type FileEntry = {
	oid: string;
	path: string;
	name: string;
	type: "blob" | "tree";
	depth: number;
};

export type GitBundleState =
	| { phase: "idle" }
	| { phase: "loading"; progress: string }
	| { phase: "error"; error: string }
	| {
			phase: "ready";
			headCommit: string;
			files: FileEntry[];
			readFile: (oid: string) => Promise<Uint8Array>;
	  };

const BINARY_EXTENSIONS = new Set([
	"png",
	"jpg",
	"jpeg",
	"gif",
	"webp",
	"bmp",
	"ico",
	"tiff",
	"avif",
	"pdf",
	"zip",
	"tar",
	"gz",
	"bz2",
	"xz",
	"rar",
	"7z",
	"wasm",
	"exe",
	"dll",
	"so",
	"dylib",
	"class",
	"o",
	"a",
	"mp3",
	"mp4",
	"ogg",
	"wav",
	"avi",
	"mov",
	"mkv",
	"ttf",
	"otf",
	"woff",
	"woff2",
	"eot",
	"bundle",
]);

export function isBinaryPath(path: string): boolean {
	const ext = path.split(".").pop()?.toLowerCase() ?? "";
	return BINARY_EXTENSIONS.has(ext);
}

export function useGitBundle(bundleUrl: string | null): GitBundleState {
	const [state, setState] = useState<GitBundleState>({ phase: "idle" });

	useEffect(() => {
		if (!bundleUrl) {
			setState({ phase: "idle" });
			return;
		}

		let cancelled = false;
		const abortController = new AbortController();

		const load = async () => {
			setState({ phase: "loading", progress: "Downloading bundle…" });

			try {
				const response = await fetch(bundleUrl, {
					signal: abortController.signal,
				});
				if (!response.ok) {
					throw new Error(`HTTP ${response.status}: ${response.statusText}`);
				}
				const buffer = await response.arrayBuffer();
				if (cancelled) return;

				setState({ phase: "loading", progress: "Parsing bundle header…" });

				const bytes = new Uint8Array(buffer);
				const { headCommit, packOffset } = parseBundleHeader(bytes);
				if (!headCommit) throw new Error("Bundle contains no HEAD commit");
				if (
					typeof packOffset !== "number" ||
					packOffset < 0 ||
					packOffset >= bytes.length
				) {
					throw new Error("Invalid bundle pack offset");
				}

				const packData = bytes.slice(packOffset);
				if (cancelled) return;

				// Pure in-memory filesystem — no IndexedDB, safe for sandboxed iframes.
				const fs = createMemFs();

				try {
					await git.init({ fs: fs as never, dir: "/" });
				} catch (cause) {
					if (!(cause instanceof Error && (cause as NodeJS.ErrnoException).code === "EEXIST"))
						throw cause;
				}

				await fs.promises.mkdir("/.git/objects/pack", { recursive: true });
				await fs.promises.writeFile("/.git/objects/pack/pack.pack", packData);

				if (cancelled) return;
				setState({ phase: "loading", progress: "Indexing pack file…" });

				await git.indexPack({
					fs: fs as never,
					dir: "/",
					filepath: "objects/pack/pack.pack",
				});

				// Write detached HEAD so resolveRef works
				await fs.promises.writeFile("/.git/HEAD", `${headCommit}\n`);

				if (cancelled) return;
				setState({ phase: "loading", progress: "Walking repository tree…" });

				type RawEntry = {
					oid: string;
					path: string;
					name: string;
					type: "blob" | "tree";
					depth: number;
				};

				const walked = (await git.walk({
					fs: fs as never,
					dir: "/",
					trees: [git.TREE({ ref: headCommit })],
					// eslint-disable-next-line @typescript-eslint/no-explicit-any
					async map(filepath: string, [entry]: any[]) {
						if (!entry || filepath === ".") return null;
						const type = (await entry.type()) as string;
						if (type !== "blob" && type !== "tree") return null;
						const oid = (await entry.oid()) as string;
						const parts = filepath.split("/");
						return {
							oid,
							path: filepath,
							name: parts[parts.length - 1],
							type: type as "blob" | "tree",
							depth: parts.length - 1,
						} satisfies RawEntry;
					},
				})) as (RawEntry | null)[];

				if (cancelled) return;

				const files = walked.filter((e): e is RawEntry => e !== null);

				// Sort: within each directory, alphabetically (trees before blobs by name)
				files.sort((a, b) => {
					const aParts = a.path.split("/");
					const bParts = b.path.split("/");
					for (let i = 0; i < Math.min(aParts.length, bParts.length); i++) {
						if (aParts[i] !== bParts[i]) return aParts[i].localeCompare(bParts[i]);
					}
					return aParts.length - bParts.length;
				});

				const readFile = async (oid: string): Promise<Uint8Array> => {
					const { blob } = await git.readBlob({ fs: fs as never, dir: "/", oid });
					return blob;
				};

				setState({ phase: "ready", headCommit, files, readFile });
			} catch (cause) {
				if (cancelled) return;
				if (cause instanceof Error && cause.name === "AbortError") return;
				setState({
					phase: "error",
					error: cause instanceof Error ? cause.message : "Failed to load bundle",
				});
			}
		};

		void load();
		return () => {
			cancelled = true;
			abortController.abort();
		};
	}, [bundleUrl]);

	return state;
}
