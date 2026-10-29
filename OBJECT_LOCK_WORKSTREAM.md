# Object Lock workstream

This branch starts from Garage v2.5.0, commit
`586726edb0170bb46f604e84fba09e5659610649`.

**This branch does not implement S3 Object Lock. Do not use it as WORM storage.**

The first change rejects unsupported Object Lock headers and browser POST option
fields before the file part with `501 NotImplemented`. Ordinary data writes must
not appear to acknowledge a retention or legal-hold promise that the storage
engine cannot enforce. The guard applies to incoming headers before dispatch,
including PUT, COPY, bucket creation, multipart and governance-bypass requests.
It does not make retention metadata, version history or garbage collection safe.

## Required architecture

Current object merge/pruning (`src/model/s3/object_table.rs`) can discard older
versions and enqueue their associated version/block cleanup. Existing lock route
names are not implementation evidence. The upstream
`doc/book/reference-manual/known-issues.md` documents the limitation of the
existing no-consensus model for safe locking/WORM.

A complete implementation needs:

1. An ordered metadata authority for protection changes and delete decisions,
   including quorum, fencing, recovery and clock assumptions. A self-contained
   Rust consensus service is the proposed direction; external PostgreSQL is an
   alternative with a different deployment model. Neither is implemented here.
2. Durable scalable version history, version-specific GET/HEAD/COPY/DELETE,
   delete markers and version listings. No implicit pruning of protected data.
3. Bucket defaults, retention modes, legal holds, permissions and an explicit S3
   compatibility profile. Newer event-hold semantics require a separate scope
   decision and must not be advertised without implementation.
4. Protection-aware PUT/COPY/multipart, lifecycle, admin, repair, replication and
   garbage collection paths. Every physical deletion needs current fenced
   authority; a stale replica must not reclaim protected blocks.
5. Feature activation and old-binary barriers, migration, backup/restore and
   downgrade restrictions, plus partition/crash/concurrency/protocol tests.

An HTTP proxy or a retention timestamp alone cannot prove these invariants.
Do not enable protected storage until the whole deletion and recovery path has
been qualified. Continue following upstream development and retire fork changes
only after tagged-release conformance and migration testing.

## Verification commands

The narrow header/field classifier has three unit tests:

```sh
cargo test --locked -p garage_api_s3 --lib protection --no-default-features
```

Four integration tests exercise the actual server and verify rejected protected
PUT/DELETE, COPY/multipart, bucket enablement and POST form mutations leave data
unchanged or create no destination:

```sh
cargo test --locked -p garage --test integration \
  --no-default-features --features bundled-libs,lmdb protection
```

Run the existing ordinary upload tests as regression coverage. These tests cover
the fail-closed guard only, not distributed retention correctness. On paths
containing `&`, bundled libsodium's configure step needs a `CARGO_TARGET_DIR`
without that character. Build dependencies remain pinned in the upstream
`Cargo.lock`; this patch does not update them.

Verified on 2026-10-29, macOS arm64, Rust 1.99.1: three classifier unit tests,
four new protection integration tests, the existing simple PUT/GET test, and
three existing browser POST tests passed. The POST protection test also passed
again with the POST regression group (nine integration executions, eight unique
integration tests). Linux and distributed conformance were not tested.

The test SDK client now uses the fixture's configured endpoint instead of always
using its default port. Temporary test servers were explicitly cleaned up after
each run because the macOS fixture can leave its server child running on exit.
