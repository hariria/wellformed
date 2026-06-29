# Changesets

TypeScript package release notes and versioning are managed with Changesets.

## Add a Changeset

From `typescript/`:

```bash
pnpm changeset
```

Choose the package (`wellformed`), bump type, and provide a concise summary.

## Apply Versions

```bash
pnpm version-packages
```

This updates package versions and changelog entries.

## Publish

```bash
pnpm release-packages
```
