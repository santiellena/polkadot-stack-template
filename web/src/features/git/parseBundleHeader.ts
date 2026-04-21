export type BundleHeader = {
	refs: Map<string, string>; // refname → sha hex
	headCommit: string | null;
	packOffset: number;
};

export function parseBundleHeader(bytes: Uint8Array): BundleHeader {
	const decoder = new TextDecoder("utf-8", { fatal: false });
	const refs = new Map<string, string>();
	let headCommit: string | null = null;
	let offset = 0;

	function readLine(): string | null {
		if (offset >= bytes.length) return null;
		const start = offset;
		while (offset < bytes.length && bytes[offset] !== 0x0a) offset++;
		const line = decoder.decode(bytes.slice(start, offset));
		if (offset < bytes.length && bytes[offset] === 0x0a) offset++;
		return line;
	}

	const firstLine = readLine();
	if (!firstLine?.startsWith("# v")) {
		throw new Error("Not a valid git bundle: missing version header");
	}
	const isV3 = firstLine.includes("v3");

	// v3 has a capabilities block terminated by a blank line before the refs section
	if (isV3) {
		while (true) {
			const line = readLine();
			if (line === null) throw new Error("Unexpected end of bundle header (v3 caps)");
			if (line === "") break;
		}
	}

	// Read prerequisites (-sha ref) and references (sha refname)
	while (true) {
		const line = readLine();
		if (line === null) throw new Error("Unexpected end of bundle header");
		if (line === "") break; // blank line ends the header

		if (line.startsWith("-")) continue; // prerequisite, skip

		const spaceIdx = line.indexOf(" ");
		if (spaceIdx === 40 || spaceIdx === 64) {
			const sha = line.slice(0, spaceIdx);
			const refname = line.slice(spaceIdx + 1);
			refs.set(refname, sha);

			// Prefer main/master, then any branch, then anything
			if (headCommit === null) headCommit = sha;
			if (refname === "refs/heads/main" || refname === "refs/heads/master") {
				headCommit = sha;
			}
		}
	}

	// Verify PACK magic at current offset; scan forward as fallback
	const PACK = [0x50, 0x41, 0x43, 0x4b];
	if (
		offset + 4 <= bytes.length &&
		bytes[offset] === PACK[0] &&
		bytes[offset + 1] === PACK[1] &&
		bytes[offset + 2] === PACK[2] &&
		bytes[offset + 3] === PACK[3]
	) {
		return { refs, headCommit, packOffset: offset };
	}

	for (let i = offset; i < bytes.length - 4; i++) {
		if (
			bytes[i] === PACK[0] &&
			bytes[i + 1] === PACK[1] &&
			bytes[i + 2] === PACK[2] &&
			bytes[i + 3] === PACK[3]
		) {
			return { refs, headCommit, packOffset: i };
		}
	}

	throw new Error("PACK data not found in bundle");
}
