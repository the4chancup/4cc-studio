# Citation audit rulings

The lead's verdict on each flag of the citation audit (`scripts/citation_audit.py`,
CONTRIBUTING.md "Citation audit"), one row per flag, newest last. A flag is a test citing an
acceptance scenario that SubQ Snap judges it does not prove (P under 0.3). The key hashes the
pair's state (the scenario's text, the test, its same-file helpers), so a ruled pair is not
raised again until one of them changes. The audit's report prints each new flag's row with the
verdict and reason left to fill in. Accepted: the scenario does not state the case the test
checks, and the fix (a widened scenario, a new one, or the test's citation moved) is named in
the reason. Rejected: the test proves the scenario as written. A `|` inside a reason is
written `\|`.

| Key | Verdict | Date | Scenario | Test | P | Reason |
|---|---|---|---|---|---|---|
