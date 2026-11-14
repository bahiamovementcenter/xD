# Gordian Envelope in xD: where it could fit

Status: research, 2026-11-14. Nothing here is decided; decisions go in ADRs.

## Why this note

Ash's aim is a fully deployed xD system whose content lives publicly, with
extensions that keep private whatever they like, or share it with chosen
people. Ash pointed at [Gordian Envelope][rust] (Blockchain Commons) and
asked (2026-11-14) where xD could use it: not only for content, but in the
app and on the operating system.

xD has met Envelope before. ADR 0001 chose Rust partly because it has "the
strongest libraries for content digests and Gordian Envelopes", E4 lists
Envelope digest trees as prior art for a content digest, and E10 ends with
"this is a Gordian Envelope: identity as the subject, the rest as
assertions, each one separately elidable or encryptable". This note follows
that thread through every level, says what fits the core and what belongs
in an extension, and recommends a first step.

The project's rules frame every answer:

- No general mechanism until two real uses, and a use counts once it is a
  config (extensibility.md, ADR 0007).
- Behaviours live in the library; JD and PARA set them as overridable
  defaults (ADR 0005 decision 1).
- Units stay independent and stack with `&`.
- Ash's own content and layout belong in Ash's personal project, not in
  xD's defaults.
- The core does no I/O and never runs a configuration's functions.

## 1. What Envelope is, in xD's terms

Facts below are from the [IETF draft][draft] (draft-mcnally-envelope-12,
2026-08-10), the [Rust crate][rust] (0.43.0, February 2026) and the
[developer site][site], read 2026-11-14.

**Shape.** An envelope is a *subject* with zero or more *assertions*, each a
*predicate* and an *object*. Every part is itself an envelope, so anything
can carry assertions about it. Encoding is [deterministic CBOR][dcbor]
(dCBOR): one byte sequence per value, Unicode NFC text, sorted map keys.

**Digest tree.** Every part has a SHA-256 digest. A leaf's is the hash of
its dCBOR bytes; a node's is the hash of its subject's digest followed by
its assertions' digests in sorted order; an assertion's hashes predicate and
object. So the whole envelope has one root digest, and the subject has its
own digest that ignores the assertions.

**Operations that keep the digest tree.** Any holder, not only the author,
can:

| Operation | Does | Keeps signatures? |
| --- | --- | --- |
| Elide | replace any part with its digest | yes |
| Encrypt | replace a part with ciphertext (ChaCha20-Poly1305) that carries the part's digest | yes |
| Compress | replace a part with its compressed form | yes |
| Reveal (inclusion proof) | hand over an elided part; its digest proves it was there | yes |

**Operations that add to it.**

| Operation | Does |
| --- | --- |
| Salt | add random data to a part, so a guessable value ("Taxes") can't be found by hashing guesses |
| Sign | assert a signature over a part's digest (Ed25519, Schnorr, SSH keys; post-quantum options listed) |
| Permits | wrap the content key for one or more readers: a public key, a password, SSKR shares, a FROST group |
| SSKR | split a key into Shamir shares, by groups and thresholds |
| Known values, attachments, edges, expressions | small integers for common predicates; vendor data; links between envelopes; function calls and requests |

**Around it**, in the same organisation:

| Piece | Is |
| --- | --- |
| [XID][xid] | a stable 32-byte identifier: the hash of an inception public key. It resolves to an envelope (the XID document) listing current keys, devices and delegations, so keys can rotate while the XID stays. Rotation and revocation details are in BCR-2024-010, not yet read for this note. |
| [Provenance marks][provemark] | a hash chain in which each mark commits to the next: proves an ordered series (editions, versions) with no gaps, without trusted clocks |
| [Gordian Clubs][clubs] | a group publishing encrypted *editions*, chained by provenance marks; readers hold permits; writers sign, alone or as a FROST threshold |
| [Hubert][hubert] | a write-once dead drop for encrypted messages over the BitTorrent DHT or IPFS; a proof of concept |
| GSTP | sealed requests and responses between parties, over any channel |
| `envelope` CLI | `cargo install bc-envelope-cli`: build, format, elide, encrypt, sign, split |

**Implementations.** Envelope in Rust, Swift and TypeScript; dCBOR also in
Ruby. XID and provenance marks in Rust and TypeScript, Clubs and Hubert in
Rust only. No Go, no Nickel, no WASM build listed. Licence
BSD-2-Clause-Patent.

**Maturity.** Both drafts are individual submissions, not IETF-endorsed;
Envelope's intended status is Experimental. The Rust README calls the crate
"production-ready" in one place and says it "should not be used for
production tasks until it has received further testing and auditing" in
another. The crate is pre-1.0 and its changelog has breaking renames.

### The one property nothing else gives xD

Encryption at rest (age, git-crypt, sops), signatures (SSH, minisign),
Merkle digests (Git trees) and capability URLs (Tahoe-LAFS) each exist
alone. What Envelope adds is that **a holder can remove or hide parts of a
signed record and the signature still verifies**. That is exactly "content
lives publicly, extensions keep private what they like": one signed record
of an entry, and several views of it (public, family, only me) that are all
provably the same record.

## 2. Content level

### C1. The entry record (E10's four slots)

E10 splits an entry into identity, label, pointer and proof, each a set
with one primary. As an envelope:

```
"11.13" [                              ← subject: the identity
  'isA': 'xd:Entry'
  'label': "Taxes"                     ← primary label
  'label': "Impostos" ['lang': "pt"]
  'alias': "k3f9q2m1x0"                ← hidden alias, ADR 0006 decision 7
  'state': 'live'                      ← lifecycle, ADR 0005
  'topic': 'money'                     ← facets
  'location': "git+https://…?dir=…"    ← ADR 0008 locations
  'digest': sha256:…                   ← E4 proof
  'signed': Signature
]
```

Every row is separately elidable. A public view elides `'label'` and
`'location'` for a private entry and keeps `'state'`; a family view
decrypts them. **Fit: high.** This is the format E10 already named, and it
costs nothing on disk because it is an export of what the index holds.

### C2. The content digest (E4) as an Envelope digest

E4 wants a Merkle hash per entry and per level, and leaves open whether
labels are in it. Envelope answers that question by having both: the
*subject digest* covers the identity alone, and the *envelope digest*
covers the identity with every assertion, label included. Defining E4's
tree as envelopes (a folder is a subject with one assertion per child
name, whose object is the child's digest; a file is a leaf holding its
SHA-256) also gives E4's sync story (compare root digests, descend where
they differ) and adds **inclusion proofs**: "this file was in `11.13` at
version X" without revealing the rest of the tree.

File bytes never go inside the envelope; a file is represented by its hash,
as a Git tree holds blob ids. **Fit: high, and cheapest to decide now**,
while E4 has no implementation (section 6).

### C3. The hidden identity's birth hash (ADR 0006)

ADR 0006 takes a hash of `title`, `content`, `created` and a `nonce` once,
at creation, and freezes it, but doesn't say what bytes are hashed. If the
birth hash is the digest of a **salted birth envelope**:

```
'birth' [ 'title': "Website redesign"  'content': sha256:…
          'created': 2026-11-13T23:07:52Z  'salt': … ]
```

then the identity is still an opaque, frozen SHA-256 (ADR 0006 already
picked `'sha256` and the `sha256:` prefix), the nonce is Envelope's salt,
and the owner can later **prove** where an identity came from by revealing
the birth envelope, or any one field of it. Nobody else can guess it from
the title, because of the salt. **Fit: high.** It changes no decision in
ADR 0006, only fills in its byte format.

### C4. Private content inside a public tree

A private file in a public repository is the case Ash describes. Options:

| Approach | What is public | Notes |
| --- | --- | --- |
| Leave it out (`.gitignore`, a separate root) | nothing, not even that it exists | simplest; placement (ADR 0005) can already send private entries to another root |
| Encrypt the file (age, git-crypt, sops) | ciphertext, file name | mature tools; no structure |
| Encrypt as an envelope with permits | ciphertext, the digest, chosen assertions | per-reader permits; the same digest the public record signs |
| Elided record only | the entry exists, with its digest | nothing to decrypt; the content lives elsewhere |

Envelope fits the **record** better than the **bytes**: large files are
better served by age or rclone's `crypt` (which the storage design already
mentions, `encrypt = true`), with the envelope holding the ciphertext's
digest and the permits for its key. **Fit: medium**; it is one storage
backend among several.

### C5. Work packages, notes and the JDex

- A **work package** is an entry like any other (C1). Its transitions
  (`live → archived`, ADR 0005) could be signed assertions, so a public
  view shows that `W0012` was finished, and by whom, without its contents.
- **Notes** in the JDex (Markdown, Org) stay plain text on disk. Their
  export is C1: an envelope per entry with the note as an elidable,
  encryptable object. xD never rewrites the note itself (ADR 0006 already
  rules out writing metadata into files).
- **Ownership** (E11's `.owners`): owners as XIDs (O3) make an owner file
  verifiable, and an approval a signature.

## 3. App level

### A1. Plans and receipts (ADR 0003)

`xd check` already returns a receipt, the plan's hash, and `xd apply`
refuses a mismatched one. A plan as an envelope makes the receipt its
digest and **approval a signature**: an agent writes a plan, a person (an
approver in E11's sense) signs its digest, and `apply` checks the signature
as well as the hash. That is the agent loop's trust boundary written down.
**Fit: medium-high**, and it has two uses inside xD's own design already
(agent plans, and owner approvals in shared systems).

### A2. The journal and applied history

`.xd/plans/` and the journal are local and trusted. jj already holds
history. Provenance marks would add a chain of applied plans that another
machine can verify has no gaps, which matters only once two writers share a
system (ADR 0002's deferred case). **Fit: low now**, later with shared
systems.

### A3. Publishing: a "public view" of a system

The app-level heart of Ash's aim: `xd` renders a system's records (C1) and
elides or encrypts each by a rule, then writes the result somewhere public.
In xD's existing terms this needs **no new core mechanism**:

- **Visibility is a facet** (ADR 0005 decision 2): `visibility = xd.facet {
  values = { public, shared, private }, cardinality = 'one, inherit = 'down
  }`, set on an area and overridden per category or ID, by the same
  inheritance rule as everything else. An `audience` facet (`'many`) names
  who `shared` means.
- **Publication is a location** (ADR 0008): a place the entry also lives,
  bound to an adapter, whose *condition* follows the entry's facets
  (ADR 0008 decision 3), the way a repository's archived flag follows
  state. The adapter writes the elided or encrypted envelope.
- The adapter is Rust (`bc-envelope`), on the app-adapter seam, in a unit of
  its own (say `nickel/disclosure.ncl` plus an `xd-disclosure` crate). JD
  and PARA don't import it; a consumer who wants it stacks it with `&`.

So privacy is an **extension built from core mechanisms**, which is what the
project rules ask for. The new parts are the disclosure adapter and the
envelope export format, nothing in identity or numbering.

### A4. The Nickel library and `xd.ncl`

Nickel cannot compute Envelope digests. `std.hash` hashes *strings* with
SHA-256, but Nickel has no byte strings and no CBOR serializer, and a leaf's
digest is over dCBOR bytes. That fits ADR 0005's split anyway: Nickel
declares the policy (which facets, which audiences, which keys by name),
and Rust computes and signs, handing back resolved values the way
placement is resolved. Contracts can still check what is declarative:
an audience names a known key, a `public` entry has no private-only facet.

Signing **configurations** themselves (a released `jd.ncl`, a preset
someone depends on) is possible but weak: consumers pin the `xd` Nickel package by
commit (ADR 0007, Consequences), and a Git commit with an SSH
signature already proves the same thing. **Fit: low.**

### A5. The capability registry

`nickel/capabilities.ncl` is public data in a public repository; it needs
no privacy or proofs. **Fit: none**, except as the place a `disclosure`
capability is recorded (section 6).

### A6. Sync and the agent tools

- **Sync between machines** benefits from C2's digest tree (compare roots,
  descend) more than from any envelope feature; Envelope's diff and merge of
  envelopes is a later convenience.
- **The xjd mod and agents** could exchange plans as GSTP requests and
  responses, sealed to the user's key. Only worth it once an agent runs on
  a machine the user doesn't control; today the plan is a local file.
- **Hubert** could carry encrypted plans or editions between devices with
  no server. It is a proof of concept; a Git remote does the same job now.

## 4. Operating-system level

### O1. Files on disk

ADR 0006 fixes the marker format: "always a Nickel record, so any host that
embeds Nickel can read it". Envelope must not replace it. The marker can
carry an envelope's digest (`birth = "sha256:…"`, C3) or a `ur:envelope/…`
string as an ordinary field, so a tool that knows Envelope can verify it
and one that doesn't loses nothing. A binary `.envelope` sidecar per entry
is possible but adds a second hidden file ADR 0006 tried to avoid.
**Recommendation: envelopes are an export and a transport, never the
on-disk source of truth.**

### O2. Git and forges (ADR 0008)

- **Signing keys are already there.** Envelope signs with SSH keys, and
  GitHub verifies SSH-signed commits with the keys a user registered. One
  SSH key can sign commits, tags and xD records, and a forge already
  publishes the public half. That is the cheapest identity xD can rely on.
- **Public repository, private entries**: C4, written through the
  publication location (A3), with locations keeping their URIs (ADR 0008
  decision 2b) as assertions.
- **History never forgets.** A file committed in plaintext stays in Git
  history after it is encrypted. The disclosure adapter must encrypt before
  the commit, and `check` should refuse a plan that would commit a
  `private` entry's content unencrypted to a location marked public. This
  is the single most important safety rule of the whole design.

### O3. Identity and keys

E10 leaves the owner level ("owner → system → AC.ID") for later, with
`did:key` as its first option. An XID is the better fit for xD:

| | `did:key` | XID |
| --- | --- | --- |
| Stable across key changes | no: the key *is* the identifier | yes: the hash of the first key; later keys listed in its document |
| Devices | one key | one key per device in the document, each removable |
| Private parts | none | the document is an envelope, so devices or delegations can be elided |
| Registry | none | none (self-certifying) |
| Ecosystem | wide (W3C DID) | Blockchain Commons only |

So `xid:…/P76/11.11` would be a globally unique name for an entry that
survives the owner changing keys. **Fit: high, when the owner level is
needed**; E10 already says that waits until a system is shared.

**Where keys live** is an OS question: an SSH agent, the OS keychain, a
hardware key. xD should never hold a private key in its own files, as
xdg-agents never prints an agent's credentials. Envelope adds SSKR for
**recovery**: the system key split 2-of-3 across devices or trusted
people, so losing a laptop doesn't lose every private record.

### O4. Devices

Each device gets its own key, listed in the owner's XID document. A lost
device is removed from the document; records it signed before stay valid,
records it signs after don't. Provenance marks order a device's edits
without trusting its clock. This only matters once more than one device
writes; until then one key is enough.

### O5. Sharing with chosen people

| Need | Envelope answer |
| --- | --- |
| Share one entry with one person | encrypt to their public key (a permit) |
| Share an area with a group, with updates | a Gordian Club: each update an edition, readers hold permits |
| Share with "any 2 of these 3" | SSKR or FROST permits |
| Prove something without showing it | elide everything else, keep the signature |
| Stop sharing | **forward only**: the next edition leaves them out; what they already decrypted stays theirs |

Revocation is the honest limit of every encryption-based design, not of
Envelope alone: nothing takes back ciphertext someone could already
decrypt. Rotating the content key on every edition makes removal effective
from then on.

## 5. Core, extension, or neither

| Use | Level | Where it belongs | When |
| --- | --- | --- | --- |
| C2 digest tree as Envelope digests | content | **core decision**: the format of E4's digest | now, as a choice of format; code when E4 is built |
| C3 birth hash as a salted envelope digest | content | **core decision**: fills ADR 0006's byte format | now, in ADR 0006 while it is proposed |
| C1 entry record as an envelope | content | export format of the index, in an extension crate | with A3 |
| A3 visibility facet and publication location | app | **extension unit** (`disclosure.ncl` + Rust adapter) on ADR 0005 facets and ADR 0008 locations | after two configs |
| A1 signed plans | app | extension to the agent loop | with a second writer or a shared system |
| O3 XID owners, O4 devices | OS | extension; E10's owner level | when a system is shared |
| O5 sharing, Clubs | OS | extension, on top of A3 | after A3 |
| C4 encrypted bytes | content | a storage backend (age, rclone `crypt`), envelope holds the digest | when a storage backend is built |
| A2 provenance chain, A6 GSTP and Hubert | app | none yet | revisit with shared systems |
| A4 signed configs, A5 registry | app | none | Git signatures cover it |
| O1 envelopes as on-disk markers | OS | **no**: markers stay Nickel (ADR 0006) | |

Only two rows touch the core, and neither adds a mechanism: both choose the
byte format of a hash xD has already decided to compute. Everything that
makes things private is an extension built on facets and locations, which
fits "privacy is an extension, not core".

**Two consumers.** By ADR 0007, `disclosure` gets built once two configs
need it. Ash's public system with private parts is the first, written in
Ash's personal project. Candidates for the second: a family or team
sharing one area of a JD system with outsiders; a PARA vault whose
`Resources` are published and whose `Projects` aren't; a research archive
with embargoed entries. Each is a `known` system until someone writes it
as a config.

## 6. Risks

| Risk | Detail | Mitigation |
| --- | --- | --- |
| Maturity | Unaudited, pre-1.0, breaking renames between minor versions; drafts are Experimental and individual | Keep it behind an extension crate; pin the version; the core depends only on SHA-256 and a documented byte format |
| Spec drift | dCBOR and Envelope may change before any RFC | C2 and C3 depend only on the digest construction, which the draft calls stable; changing the hash needs a new tag, so old digests stay readable |
| Rust-only for xD's purposes | Rust, Swift, TypeScript; no Go, no Nickel, no WASM listed | xD is Rust, so the core is fine. Nickel declares policy only (A4). Go tools (xdg-agents, a future go-ncl) would read exports through xD's CLI, not compute digests |
| Nickel engine can't verify | Nickel has `std.hash` on strings but no bytes or CBOR | Rust resolves, Nickel declares, as for placement |
| Key loss | Losing the key loses every private record | SSKR shares across devices or people; keys in the OS keychain or hardware |
| Revocation | Shared ciphertext can't be taken back | Forward-only removal by key rotation per edition; say so in the UI |
| Leaks through structure | An elided part still shows that it exists and its digest; one signature correlates every view; a JD number itself reveals the tree (`11.13` under a health area) | Salt every private leaf; elide whole assertions, not only values; publish hidden aliases (ADR 0006) instead of numbers for private entries; per-audience salts where views must not be linked |
| Git keeps plaintext | A private file committed once stays in history | `check` refuses a plan that commits private content to a public location unencrypted (O2) |
| Complexity | Several specs, keys and permits for what may start as "don't publish that folder" | Start with placement to a non-public root (C4's first row) and add envelopes only when proofs or selective sharing are needed |
| Ecosystem lock-in | XID and Clubs are used mostly by Blockchain Commons | Keep identifiers as plain strings in records; an owner could be a `did:key` or a domain instead of an XID |

## 7. Prior art

- **Perkeep (Camlistore)**: a *permanode* is a random, signed object whose
  identity never changes, and every change is a signed claim about it. This
  is ADR 0006's hidden identity plus C1's assertions, in production since
  2011; worth reading before designing claims.
- **Git**: tree hashes (E4), SSH-signed commits (O2).
- **git-crypt, sops, age, rclone `crypt`**: encryption at rest for files in
  repositories and remotes (C4).
- **Obsidian Publish, Quartz, Hugo drafts**: publishing a chosen subset of
  a notes vault, by folder or front matter (A3); none can prove the public
  part belongs to a larger signed whole.
- **SD-JWT, W3C Verifiable Credentials with BBS+**: selective disclosure of
  signed claims; credential-shaped, not document-shaped.
- **Certificate Transparency, Sigstore**: append-only Merkle logs with
  inclusion proofs (A2).
- **Tahoe-LAFS, Peergos, Keybase KBFS**: capability-based encrypted
  sharing with forward-only revocation (O5).
- **Solid pods**: per-resource access control for public and private data
  under one identity (O3, O5), server-enforced rather than cryptographic.

## 8. Recommended first step

**Decide the digest format, then prove it with a throwaway spike.**

1. **A short amendment to ADR 0006 (and E4's entry in jd-experiments):**
   the birth hash is the Envelope digest of a salted birth record (C3), and
   E4's content digest is an Envelope digest tree (C2). Both stay SHA-256
   with the `sha256:` prefix ADR 0006 already chose. This is the only step
   that is cheap now and costly later: once markers hold birth hashes, their
   byte format can't change without re-identifying every entry. It adds no
   mechanism and no dependency to the core (the construction is a few lines
   over dCBOR, or the `bc-envelope` crate behind a feature).
2. **A spike, not merged:** export one small JD tree's index as envelopes,
   elide entries marked private, sign with an SSH key, and verify the
   public view with the `envelope` CLI. It answers in a day what this note
   can't: how Nickel's exported values map to dCBOR, how big a public view
   of a real system is, and whether an elided tree leaks more than expected.
3. **Then the ADR 0007 procedure:** write Ash's public-with-private system
   as a config in Ash's personal project, using a `visibility` facet and a
   publication location. It fails with a `disclosure` capability code,
   which is registered with that one config and waits for a second.

Steps 1 and 2 are independent; step 3 waits for the personal project's
config.

## Open

- XID rotation, revocation and delegation as BCR-2024-010 specifies them
  (not read for this note).
- Whether a private entry's public record should show its JD number, or
  only a hidden alias (section 6, structure leaks).
- Whether the digest tree includes file names of private files (they leak
  in an elided tree unless each name is its own salted, elided part).
- How E4's digest treats symlinks, empty folders and `'path` storage.

[rust]: https://github.com/BlockchainCommons/bc-envelope-rust
[draft]: https://datatracker.ietf.org/doc/draft-mcnally-envelope/
[dcbor]: https://datatracker.ietf.org/doc/draft-mcnally-deterministic-cbor/
[site]: https://developer.blockchaincommons.com/envelope/
[xid]: https://developer.blockchaincommons.com/xid/
[provemark]: https://developer.blockchaincommons.com/provemark/
[clubs]: https://developer.blockchaincommons.com/clubs/
[hubert]: https://developer.blockchaincommons.com/hubert/
