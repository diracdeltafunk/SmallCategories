# Website metadata export

This directory contains the public, database-derived data needed to build the static SmallCats website without Supabase:

- `propositions.json`: proposition IDs, names, descriptions, and stable bit assignments;
- `categories.ndjson`: category UUIDs, public names/descriptions, multiplication-table fingerprints, and compact proposition masks;
- `export-manifest.json`: format version, export timestamp, and checked source counts.

The category tables themselves remain canonical in `../database/`. The website compiler matches each exported row to a category by the SHA-256 hash of its compact JSON multiplication table. It does not trust the historical numeric index.

These fields were already served publicly by smallcats.info. Connection strings, API keys, passwords, roles, private schemas, and other credentials are not included.

Regenerate this directory with `SmallCategories-site/static-site/scripts/export_supabase.py` while the source database is available, validate a complete static build, and replace all three data files together. The export format is documented and checked by the static-site compiler.
