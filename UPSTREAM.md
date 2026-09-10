# Upstream provenance

Openarb Trade SDK is maintained by Openarb and is derived from
[0xfnzero/sol-trade-sdk](https://github.com/0xfnzero/sol-trade-sdk).

- Imported upstream commit: [`e4210595e3e97b44ec7c02d461beb98868c0b5de`](https://github.com/0xfnzero/sol-trade-sdk/commit/e4210595e3e97b44ec7c02d461beb98868c0b5de)
- Upstream package version at import: `5.0.3`
- Original copyright: Copyright (c) 2024 MiracleAI-Labs
- License: [MIT](LICENSE)

The import preserves upstream Git history and the destination repository's initial
commit. The original authors remain credited in the package metadata and history.
The Openarb package retains the upstream version number for this initial import;
this does not imply that it has been published to crates.io.

## Openarb changes

- Rename the Rust package to `openarb-trade-sdk` and imports to `openarb_trade_sdk`.
- Replace the English and Chinese landing READMEs and update documentation and examples.
- Point repository and release metadata to `openarbmev/Openarb-Trade-SDK`.
- Resolve parser and streamer dependencies from crates.io at the same exact versions,
  instead of requiring sibling folders from the upstream development checkout.
- Remove the upstream Context7 registration and promotional referral parameter.
- Add a Linux CI build check and ignore local environment files.

Protocol instruction layouts, program addresses, relay endpoints, fee recipients,
and trading behavior are inherited from upstream. Third-party services and
dependencies remain separately maintained.

## Bringing in upstream updates

The initial local checkout has an `upstream` remote. For a fresh clone, add it once:

```sh
git remote add upstream https://github.com/0xfnzero/sol-trade-sdk.git
```

Fetch and review changes before merging them on a dedicated branch. Preserve
Openarb package names, repository links, and both copyright notices when resolving
conflicts. Historical documents such as `RELEASE_v4.0.2.md` describe inherited
upstream releases, not an Openarb publication.
