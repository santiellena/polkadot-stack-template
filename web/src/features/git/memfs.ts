type FsNode =
	| { kind: "file"; data: Uint8Array; mode: number; mtimeMs: number }
	| { kind: "dir"; mode: number; mtimeMs: number }
	| { kind: "symlink"; target: string; mode: number; mtimeMs: number };

function posixError(code: string, path: string): Error {
	const e = new Error(`${code}: ${path}`);
	(e as NodeJS.ErrnoException).code = code;
	return e;
}

function normalize(path: string): string {
	const parts = path.split("/").filter(Boolean);
	const out: string[] = [];
	for (const p of parts) {
		if (p === ".") continue;
		if (p === "..") out.pop();
		else out.push(p);
	}
	return "/" + out.join("/");
}

class MemFsStats {
	constructor(private n: FsNode) {}
	get mode() {
		return this.n.mode;
	}
	get size() {
		return this.n.kind === "file" ? this.n.data.length : 0;
	}
	get ino() {
		return 0;
	}
	get mtimeMs() {
		return this.n.mtimeMs;
	}
	get ctimeMs() {
		return this.n.mtimeMs;
	}
	get uid() {
		return 0;
	}
	get gid() {
		return 0;
	}
	get dev() {
		return 1;
	}
	get type() {
		return this.n.kind === "dir" ? "dir" : this.n.kind === "symlink" ? "symlink" : "file";
	}
	isFile() {
		return this.n.kind === "file";
	}
	isDirectory() {
		return this.n.kind === "dir";
	}
	isSymbolicLink() {
		return this.n.kind === "symlink";
	}
}

export function createMemFs() {
	const store = new Map<string, FsNode>();
	store.set("/", { kind: "dir", mode: 0o40755, mtimeMs: Date.now() });

	const followSymlinks = (path: string, depth = 0): string => {
		if (depth > 40) throw posixError("ELOOP", path);
		const node = store.get(path);
		if (!node || node.kind !== "symlink") return path;
		const target = node.target.startsWith("/")
			? normalize(node.target)
			: normalize(path.slice(0, path.lastIndexOf("/") + 1) + node.target);
		return followSymlinks(target, depth + 1);
	};

	// Ensure all ancestor directories exist, creating them on demand.
	// isomorphic-git's internal fs.write() swallows write errors, so failing
	// silently on a missing parent would leave objects un-written and cause
	// downstream null-dereference crashes when reading them back.
	const mkdirpSync = async (p: string) => {
		const lastSlash = p.lastIndexOf("/");
		const parent = lastSlash <= 0 ? "/" : p.slice(0, lastSlash);
		if (!store.has(parent)) await mkdirpSync(parent);
		if (!store.has(p)) {
			store.set(p, { kind: "dir", mode: 0o40755, mtimeMs: Date.now() });
		}
	};

	const promises = {
		async readFile(
			path: string,
			opts?: { encoding?: string } | string,
		): Promise<Uint8Array | string> {
			const p = normalize(path);
			const real = followSymlinks(p);
			const node = store.get(real);
			if (!node) throw posixError("ENOENT", path);
			if (node.kind !== "file") throw posixError("EISDIR", path);
			const enc = typeof opts === "string" ? opts : opts?.encoding;
			if (enc) return new TextDecoder(enc === "utf8" ? "utf-8" : enc).decode(node.data);
			return node.data;
		},

		async writeFile(path: string, data: Uint8Array | string): Promise<void> {
			const p = normalize(path);
			// Auto-create missing parent directories (mirrors LightningFS implicit-dir behaviour).
			const lastSlash = p.lastIndexOf("/");
			const parent = lastSlash <= 0 ? "/" : p.slice(0, lastSlash);
			if (!store.has(parent)) await mkdirpSync(parent);
			if (store.has(parent) && store.get(parent)!.kind !== "dir")
				throw posixError("ENOTDIR", p);
			const bytes =
				typeof data === "string"
					? new TextEncoder().encode(data)
					: data instanceof Uint8Array
						? data
						: new Uint8Array(data as ArrayBuffer);
			store.set(p, { kind: "file", data: bytes, mode: 0o100644, mtimeMs: Date.now() });
		},

		async unlink(path: string): Promise<void> {
			const p = normalize(path);
			if (!store.has(p)) throw posixError("ENOENT", path);
			store.delete(p);
		},

		async readdir(path: string): Promise<string[]> {
			const p = normalize(path);
			const node = store.get(p);
			if (!node) throw posixError("ENOENT", path);
			if (node.kind !== "dir") throw posixError("ENOTDIR", path);
			const prefix = p === "/" ? "/" : p + "/";
			const names: string[] = [];
			for (const key of store.keys()) {
				if (key === p) continue;
				if (!key.startsWith(prefix)) continue;
				const rest = key.slice(prefix.length);
				if (rest && !rest.includes("/")) names.push(rest);
			}
			return names;
		},

		async mkdir(path: string, opts?: { recursive?: boolean } | number): Promise<void> {
			const p = normalize(path);
			const recursive =
				typeof opts === "object" && !!(opts as { recursive?: boolean }).recursive;
			if (store.has(p)) {
				if (recursive) return;
				throw posixError("EEXIST", path);
			}
			const lastSlash = p.lastIndexOf("/");
			const parent = lastSlash <= 0 ? "/" : p.slice(0, lastSlash);
			if (!store.has(parent)) {
				if (recursive) {
					await promises.mkdir(parent, { recursive: true });
				} else {
					throw posixError("ENOENT", path);
				}
			} else if (store.get(parent)!.kind !== "dir") {
				throw posixError("ENOTDIR", path);
			}
			store.set(p, { kind: "dir", mode: 0o40755, mtimeMs: Date.now() });
		},

		async rmdir(path: string): Promise<void> {
			const p = normalize(path);
			if (!store.has(p)) throw posixError("ENOENT", path);
			store.delete(p);
		},

		async stat(path: string): Promise<MemFsStats> {
			const p = normalize(path);
			const real = followSymlinks(p);
			const node = store.get(real);
			if (!node) throw posixError("ENOENT", path);
			return new MemFsStats(node);
		},

		async lstat(path: string): Promise<MemFsStats> {
			const p = normalize(path);
			const node = store.get(p);
			if (!node) throw posixError("ENOENT", path);
			return new MemFsStats(node);
		},

		async readlink(path: string): Promise<string> {
			const p = normalize(path);
			const node = store.get(p);
			if (!node) throw posixError("ENOENT", path);
			if (node.kind !== "symlink") throw posixError("EINVAL", path);
			return node.target;
		},

		async symlink(target: string, path: string): Promise<void> {
			const p = normalize(path);
			const lastSlash = p.lastIndexOf("/");
			const parent = lastSlash <= 0 ? "/" : p.slice(0, lastSlash);
			if (!store.has(parent)) await mkdirpSync(parent);
			store.set(p, { kind: "symlink", target, mode: 0o120000, mtimeMs: Date.now() });
		},
	};

	return { promises };
}

export type MemFs = ReturnType<typeof createMemFs>;
