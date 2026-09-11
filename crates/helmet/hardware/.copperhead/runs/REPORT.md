# Copperhead run report

Per-stage cost of the create pipeline, regenerated at the end of every run.
Resumed stages were already complete on entry and cost nothing this run.

| Stage | Wall | Turns | In | Out | Cache | Status |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| spec-seed | 1m54s | 9 | 1459.7k | 17.0k | 0% | ran |
| architecture | 51s | 3 | 155.8k | 4.0k | 0% | ran |
| part-selection | 2m03s | 7 | 901.4k | 15.3k | 0% | ran |
| schematic | 7m19s | 16 | 3229.7k | 70.0k | 0% | ran |
| **Total** | 12m07s | 35 | 5746.6k | 106.3k | 0% |  |

Slowest stage: **schematic** (7m19s). Most expensive: **schematic** (70.0k out tokens).
