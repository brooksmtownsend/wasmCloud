# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.15.0 (2024-05-06)

### Chore

 - <csr-id-857c9757ebaa5b835a564be5c70ac3466c01c0ca/> bump to 0.14.0
 - <csr-id-1bad246d9e174384c1a09bdff7e2dc88d911792e/> remove unused dependencies
 - <csr-id-36f0b18737f244d3f946faf8a14626dba619b931/> bump to 0.13
 - <csr-id-9c8abf3dd1a942f01a70432abb2fb9cfc4d48914/> address clippy issues
 - <csr-id-ee9d552c7ea1c017d8aa646f64002a85ffebefb8/> address `clippy` warnings in workspace
 - <csr-id-9de9ae3de8799661525b2458303e72cd24cd666f/> integrate `provider-archive` into the workspace
 - <csr-id-0b59721367d138709b58fa241cdadd4f585203ac/> integrate `wascap` into the workspace

### Documentation

 - <csr-id-05ac449d3da207fd495ecbd786220b053fd6300e/> actor to components terminology
   This change only updates documentation terminology
   to use components instead of actors.
   
   Examples will use the terminology components as well so
   I'm opting to rename the example directories now ahead
   of any source code changes for actor to component
   renames.
 - <csr-id-20ffecb027c225fb62d60b584d6b518aff4ceb51/> update wash URLs

### New Features

 - <csr-id-76c1ed7b5c49152aabd83d27f0b8955d7f874864/> support pubsub on wRPC subjects
   Up until now, publishing and subscribing for RPC communcations on the
   NATS cluster happened on subjects that were related to the wasmbus
   protocol (i.e. 'wasmbus.rpc.*').
   
   To support the WIT-native invocations, i.e. wRPC (#1389), we must
   change the publication and subscription subjects to include also the
   subjects that are expected to be used by wprc.
   
   This commit updates the provider-sdk to listen *additionally* to
   subjects that are required/used by wrpc, though we do not yet have an
   implementation for encode/deocde.

### Refactor

 - <csr-id-f5459155f3b96aa67742a8c62eb286cc06885855/> convert lattice-control provider to bindgen
   The `lattice-control` provider (AKA `lattice-controller`) enables
   easy (if not somewhat meta) control of a wasmcloud lattice, using the
   `wasmcloud-control-interface` crate.
   
   While in the past this provider was powered by Smithy contracts, in
   the WIT-ified future we must convert that contract to an WIT-ified
   interface which is backwards compatible with the smithy interface.
   
   This commit converts the `lattice-control` provider to use WIT-ified
   interfaces (rather than Smithy-based interfaces) and `provider-wit-bindgen`.
 - <csr-id-171214d4bcffddb9a2a37c2a13fcbed1ec43fd31/> use `OnceLock` to remove `once-cell`
   This commit removes the use of `once-cell` in favor of `std::sync::OnceLock`

### New Features (BREAKING)

 - <csr-id-3c56e8f18e7e40982c59ee911140cd5965c733f5/> remove capabilities
 - <csr-id-613f660a586c5b65c903161239d5f0388d534a31/> remove capability signing from wascap
 - <csr-id-42d069eee87d1b5befff1a95b49973064f1a1d1b/> Updates topics to the new standard
   This is a wide ranging PR that changes all the topics as described
   in #1108. This also involved removing the start and stop actor
   operations. While I was in different parts of the code I did some small
   "campfire rule" cleanups mostly of clippy lints and removal of
   clippy pedant mode.

### Bug Fixes (BREAKING)

 - <csr-id-93748a1ecd4edd785af257952f1de9497a7ea946/> remove usage of capability signing

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 18 commits contributed to the release over the course of 201 calendar days.
 - 16 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump to 0.14.0 ([`857c975`](https://github.com/brooksmtownsend/wasmCloud/commit/857c9757ebaa5b835a564be5c70ac3466c01c0ca))
    - Remove usage of capability signing ([`93748a1`](https://github.com/brooksmtownsend/wasmCloud/commit/93748a1ecd4edd785af257952f1de9497a7ea946))
    - Remove capabilities ([`3c56e8f`](https://github.com/brooksmtownsend/wasmCloud/commit/3c56e8f18e7e40982c59ee911140cd5965c733f5))
    - Remove capability signing from wascap ([`613f660`](https://github.com/brooksmtownsend/wasmCloud/commit/613f660a586c5b65c903161239d5f0388d534a31))
    - Remove unused dependencies ([`1bad246`](https://github.com/brooksmtownsend/wasmCloud/commit/1bad246d9e174384c1a09bdff7e2dc88d911792e))
    - Bump to 0.13 ([`36f0b18`](https://github.com/brooksmtownsend/wasmCloud/commit/36f0b18737f244d3f946faf8a14626dba619b931))
    - Actor to components terminology ([`05ac449`](https://github.com/brooksmtownsend/wasmCloud/commit/05ac449d3da207fd495ecbd786220b053fd6300e))
    - Support pubsub on wRPC subjects ([`76c1ed7`](https://github.com/brooksmtownsend/wasmCloud/commit/76c1ed7b5c49152aabd83d27f0b8955d7f874864))
    - Updates topics to the new standard ([`42d069e`](https://github.com/brooksmtownsend/wasmCloud/commit/42d069eee87d1b5befff1a95b49973064f1a1d1b))
    - Convert lattice-control provider to bindgen ([`f545915`](https://github.com/brooksmtownsend/wasmCloud/commit/f5459155f3b96aa67742a8c62eb286cc06885855))
    - Update wash URLs ([`20ffecb`](https://github.com/brooksmtownsend/wasmCloud/commit/20ffecb027c225fb62d60b584d6b518aff4ceb51))
    - Address clippy issues ([`9c8abf3`](https://github.com/brooksmtownsend/wasmCloud/commit/9c8abf3dd1a942f01a70432abb2fb9cfc4d48914))
    - Use `OnceLock` to remove `once-cell` ([`171214d`](https://github.com/brooksmtownsend/wasmCloud/commit/171214d4bcffddb9a2a37c2a13fcbed1ec43fd31))
    - Merge pull request #762 from rvolosatovs/merge/wascap ([`89570cc`](https://github.com/brooksmtownsend/wasmCloud/commit/89570cc8d7ac7fbf6acd83fdf91f2ac8014d0b77))
    - Address `clippy` warnings in workspace ([`ee9d552`](https://github.com/brooksmtownsend/wasmCloud/commit/ee9d552c7ea1c017d8aa646f64002a85ffebefb8))
    - Integrate `provider-archive` into the workspace ([`9de9ae3`](https://github.com/brooksmtownsend/wasmCloud/commit/9de9ae3de8799661525b2458303e72cd24cd666f))
    - Integrate `wascap` into the workspace ([`0b59721`](https://github.com/brooksmtownsend/wasmCloud/commit/0b59721367d138709b58fa241cdadd4f585203ac))
    - Add 'crates/wascap/' from commit '6dd214c2ea3befb5170d5a711a2eef0f3d14cc09' ([`260ffb0`](https://github.com/brooksmtownsend/wasmCloud/commit/260ffb029f05b8a6b6f9dcbf6870e281569694c2))
</details>

