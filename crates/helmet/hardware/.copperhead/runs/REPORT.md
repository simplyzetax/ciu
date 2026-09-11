# Copperhead run report

Per-stage cost of the create pipeline, regenerated at the end of every run.
Resumed stages were already complete on entry and cost nothing this run.

| Stage | Wall | Turns | In | Out | Cache | Status |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| spec-seed | — | — | — | — | — | resumed |
| architecture | — | — | — | — | — | resumed |
| part-selection | — | — | — | — | — | resumed |
| schematic | — | — | — | — | — | resumed |
| layout-draft | — | — | — | — | — | resumed |
| outputs | 1m15s | 3 | 213.8k | 4.3k | 0% | ran |
| firmware | 5m18s | 7 | 641.6k | 18.4k | 0% | ran |
| devplan | 1m41s | 3 | 189.5k | 6.3k | 0% | ran |
| **Total** | 8m15s | 13 | 1044.9k | 29.0k | 0% |  |

Slowest stage: **firmware** (5m18s). Most expensive: **firmware** (18.4k out tokens).
