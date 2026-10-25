# mAI-KSeF

## Overview

A small Rust CLI for issuing monthly B2B invoices through Poland's
[KSeF 2.0](https://ksef.podatki.gov.pl/aplikacja-podatnika-ksef-20/),
driven by a template you tweak each month. Built as a thin wrapper over the
official [KSeF API](https://github.com/CIRFMF/ksef-api).

Issuing the same recurring invoice through [the Ministry of Finance's portal](https://ap.ksef.mf.gov.pl/)
each month is tedious. This tool renders an invoice from a TOML file into
[FA(3) XML](https://ksef.podatki.gov.pl/informacje-ogolne-ksef-20/struktura-logiczna-fa-3/) and submits it to KSeF
— while keeping your actual rates out of version control.

## Layout

```
ksef-core/   library: invoice model, TOML config, FA(3) XML, KSeF client
ksef-cli/    binary `ksef`: command-line front-end
template.example.toml   committed example (fake rate)
invoice.toml            your real monthly file — GITIGNORED
```

The core/CLI split is deliberate: a future MCP server or web platform becomes a
new front-end over `ksef-core`, not a rewrite.
