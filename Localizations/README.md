# Interface translations

Locales: en, fr, ar, zh-CN (Simplified Chinese), de.
`messages.tsv` columns: English source | French | Arabic | Chinese | German.
Run `python3 review/embed-localization.py` from the project folder, or use its absolute path, after editing. It rejects duplicate sources, empty entries and mismatched `{placeholder}` names, writes `catalog.json`, and updates the inline localization section of `src/ui.html`. Production has no runtime catalog requests.

The language selector saves `titan-core-language` in browser localStorage. Missing storage falls back to the browser language; invalid locale falls back to English. With storage disabled the choice applies to the session. Arabic mirrors the interface. File paths, title IDs, user/game data, logs and backend error details are not translated. Units/technical format names keep MB, GB, PKG, LZ4, ZSTD. Browser-language selection does not change the PS5 system language or backend configuration.
