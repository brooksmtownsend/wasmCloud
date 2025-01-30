# wasmcloud-proto-types

> [!WARNING]
> This crate is experimental and likely to experience breaking changes.

## Prerequisites

- proto

## rust-analyzer

It's recommended to set the following configuration option in your rust-analyzer configuration to ensure the build script runs to generate types:

```json
{
  "rust-analyzer.cargo.buildScripts.enable": true
}
```

## Considerations

- [ ] Should the generated types be output to `src/generated`, or just kept in the `OUT_DIR`?
- [ ] Should required dependencies for generated types be included in this crate and vendored, or left up to the client?
