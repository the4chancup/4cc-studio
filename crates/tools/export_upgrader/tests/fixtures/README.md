# Export upgrader fixtures

Lead-written; tests copy them into their sandbox and never write here.

## `co Spring/`, `co Spring.zip`, `co Spring.7z`

One old (AET) export in the three input forms `upgrade` reads, each archive holding the folder
inside one main folder, as the AET page packs exports (EU-IN-01). Both archives were built from
the folder with 7-Zip's `7z a`, so the three hold the same files.

- `co Note.txt`: the real `/4ccg/` Note of `4CCG Aesthetics Export VGL26` (Lab), byte for byte
  (CRLF, no final newline) except its `Team:` line, which names `/co/` (714), the acceptance
  section's test team. It has four team colors, three player kits and one goalkeeper kit, and
  a two-line "Other Notes:" section.
- `Other/readme.txt`: one line, content no step migrates (step 10's unmigrated content).
- `Thumbs.db`: eight bytes of an OLE header, an OS artifact step 1 leaves out unreported. The
  repository's `.gitignore` ignores the name except below a `tests/fixtures/` folder, so tree
  copies that honor it (the mutation runs') keep this one.
