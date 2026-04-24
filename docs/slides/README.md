# Aperio Presentation

`reveal-md` deck for the Aperio presentation. The deck is intentionally short:
problem, product, market, demo handoff, stack learnings, next steps, Q&A.

The expected live URL for the pitch is:

```text
https://aperio.dot.li/
```

## Run locally

```bash
cd docs/slides
npm install
npm run start
```

`reveal-md` starts a local server and normally opens the deck automatically. If it
does not, open:

```text
http://localhost:1948/slides.md
```

## Export static HTML

```bash
cd docs/slides
npm run export
```

The static output is written to `docs/slides/dist`.
