# Website build data

This directory contains the preserved public metadata and proposition values used to build the static SmallCats website:

- `propositions.json`: proposition names, descriptions, and stable bit assignments;
- `categories.ndjson`: public names/descriptions, multiplication-table fingerprints, and compact proposition masks;
- `export-manifest.json`: format version, snapshot timestamp, and checked record counts.

The category tables themselves remain canonical in `../database/`. The website compiler matches each snapshot row to a category by the SHA-256 hash of its compact JSON multiplication table. It does not trust the historical numeric index.

These files are build inputs, not credentials or a database backup. They contain only information already published by smallcats.info.

When updating this snapshot, replace all three data files together and run the static site's production build. The compiler validates the format, source counts, proposition masks, and multiplication-table fingerprints.
