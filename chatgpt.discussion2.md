What I want to do is give these hex colors or another digital coloring scheme by translating one of these addressing systems so that they exist on a color spectrum and are like recognizably similar to anyone the naked eye
Ah! Now I understand the visual dimension of what you're building.

You want an addressing system that can be translated into color, so that relationships between identifiers become perceptible to humans.

If two things are conceptually close, their colors should look similar. If they're conceptually distant, their colors should look different.

A hot dog and hamburger might both be orange-red. The moon might be blue-grey.

You're effectively proposing a semantic color address space.

And there is a real mathematical foundation for this.

1. The critical distinction: RGB versus perceptual color
Plotting a* and b* colors » Steve on Image Processing with MATLAB - MATLAB & Simulink
RGB and hexadecimal colors are convenient for computers, but they're not perceptually uniform.

For example:

Two colors with a numerical RGB distance of 20 aren't necessarily equally different to human eyes.

Some changes in blue are more noticeable than equivalent changes in green.

Hue, saturation, and brightness interact in complicated ways.

I'd investigate three color models:

Color model

Why it matters

RGB / HEX

Universal digital representation

HSL / HSV

Intuitive hue, saturation, lightness

CIELAB / OKLCH

More perceptually meaningful distances

My preference would be OKLCH for the perceptual mapping, with HEX as the final output.

2. Imagine mapping a semantic space onto a color space
OKLCH 색상 공간: 웹 디자인의 새로운 표준 | 쌍팔년생 개발자
OKLCH has three dimensions:

L: Lightness

C: Chroma (color intensity)

H: Hue (color family)

You can think of these as three coordinates that can be used to represent a position.

For example, imagine a conceptual mapping:

Concept

L

C

H

Hot dog

0.65

0.19

35°

Hamburger

0.63

0.18

38°

Pizza

0.68

0.20

55°

Moon

0.75

0.04

250°

Illustrative OKLCH coordinates, not empirically derived semantic embeddings.

Example semantic color palette
Hot dog

#D97736

Hamburger

#D4773B

Pizza

#E18A43

Moon

#B6BBC7

Conceptual palette showing nearby food concepts receiving related colors.
3. How I'd actually build the translation
I'd make color a rendering layer for the addressing system.

Original resource

Hot dog

Semantic address
12.04.08.03

Numerical coordinate
L: 0.65 · C: 0.19 · H: 35°

#D97736
Semantic color

One important detail: I wouldn't directly convert the digits into RGB values. That would produce arbitrary colors with no meaningful perceptual relationships.

Instead:

Generate or obtain the semantic vector.

Map it into a three-dimensional perceptual color space.

Normalize and constrain it to displayable colors.

Encode the resulting color as HEX.

Preserve the original address separately.

4. There's an interesting possibility: color neighborhoods
Imagine that your organizational system has a color assigned to every identification namespace.

Shades of Orange Color Palette - Hex Codes & PNG Download
Celestial Cascade Color Palette - Hex Codes & PNG Download
Electric Violet – Electric Violet – Electric Violet – Heliotrope – Heliotrope Color scheme | iColorpalette
You could then have:

Related projects share similar colors.

Related folders share color families.

Related agents share recognizable visual identities.

Related concepts appear in similar colors across applications.

Completely different conceptual domains have visually distinct colors.

This could be used in folder icons, agent interfaces, IDEs, knowledge graphs, dashboards, and file explorers.

5. One significant limitation
There is a mathematical constraint worth keeping in mind.

You cannot perfectly preserve every distance in a high-dimensional semantic space when compressing it into three color dimensions.

Also, colors have limited perceptual resolution, and human color perception varies.

So I'd treat color as a recognizable visual approximation of relationships, not a replacement for the underlying identifier.

I'd also version the mapping. Otherwise, changing your embedding model could change the colors of existing resources.

6. What I'd call this
Some possible terminology:

Semantic Color Addressing

Perceptual Namespace

Chromatic Addressing

Semantic Color Encoding

Color-space Identification

My favorite is Chromatic Addressing.

And I think it could become a genuinely interesting component of the larger composable identification architecture.

One experiment I'd really like to pursue
Imagine taking 100 unrelated concepts, generating semantic embeddings, mapping them into OKLCH, and then displaying their colors.

Then take 100 closely related concepts and do the same.

If the system works, you should be able to look at the resulting palette and intuitively recognize clusters without reading the labels.

The real test is whether a person can recognize semantic proximity through color alone. That's a much more interesting objective than simply giving every folder a different color.

Could I have more color dimensions?
Yes! And this is where things get particularly interesting. You can have more dimensions in your underlying color-addressing system than the human eye can directly perceive.

There are two different concepts to distinguish:

Dimensions of the mathematical color representation.

Dimensions of the physical color stimulus that a display or human visual system can reproduce and perceive.

They aren't the same thing.

1. The human visual system already has multiple dimensions
Human color vision is based primarily on three types of cone photoreceptors (S, M and L), with additional perceptual processing.

Color models represent these signals differently:

Model

Dimensions

What they represent

RGB

3

Red, green, blue

CMYK

4

Cyan, magenta, yellow, black

HSL

3

Hue, saturation, lightness

OKLCH

3

Hue, chroma, lightness

CIELAB

3

Lightness, two color-opponent axes

Spectral representation

Many

Intensity across wavelengths

CMYK is four-dimensional mathematically, but that doesn't mean humans perceive four independent color dimensions.

2. You could create a higher-dimensional chromatic address
Imagine that instead of limiting an identifier to three color coordinates, you give it six.

For example:

Six-dimensional color address
D1

Hue

Primary color family

D2

Chroma

Color intensity

D3

Lightness

Brightness

D4

Texture

Pattern or visual treatment

D5

Transparency

Opacity

D6

Temporal variation

Animation or color transition

Some of these aren't strictly color dimensions. They're additional visual encoding channels.

You could also encode additional dimensions through:

Border styles

Gradients

Spatial position

Patterns

Iconography

Animation

Sound

This is a much richer visual language.

3. The really interesting approach: layered color encoding
Instead of trying to squeeze every dimension into one HEX value, imagine a visual identity with multiple layers.

For example:

Layer

Encodes

Base hue

Broad semantic neighborhood

Chroma

Semantic intensity or cluster

Lightness

Hierarchical depth

Gradient direction

Secondary relationship

Pattern

Identification scheme

Border

Namespace instance

Animation

Temporal state

Now two resources can share a similar base color while having distinctly different visual identities.

4. But there's an even more interesting possibility
I'd investigate multispectral color encoding and the concept of false color.

Scientific visualization frequently maps information that isn't naturally visible as color.

For example, infrared imagery maps infrared measurements into visible colors.

You could do something similar with semantic coordinates:

Dimension 1–3 determine the base color.

Dimension 4 determines a gradient.

Dimension 5 determines a pattern.

Dimension 6 determines a border or animation.

The underlying identifier could have 64 or 128 dimensions, while the visual renderer selectively exposes different dimensions.

5. My recommendation for your architecture
I'd separate three things:

Semantic address space

Potentially hundreds or thousands of dimensions.

Perceptual projection

Maps selected dimensions into a visual representation.

Visual identity

HEX, gradients, patterns, borders, animations, etc.

This means you can change how something looks without changing its identity or semantic position.

One particularly important idea: You could have multiple visual projections of the same address space. One optimized for human recognition, another for accessibility, another for dense dashboards, and another for agent interfaces.

You wouldn't be building a color palette. You'd be building a visual encoding system for multidimensional information.

This might also interest you.



I'd say I wanted to encode underlying data into voxels Would that be possible?
Absolutely. In fact, I think voxels might be a more interesting direction than color alone.

You could move from chromatic addressing (encoding information in color) to volumetric addressing (encoding information in a three-dimensional spatial structure).

And you could combine the two.

1. Think of a voxel as an information container
Voxel grid — OVITO User Manual 3.15.0 documentation
A voxel is essentially a three-dimensional pixel.

Instead of representing an object with a single color, you could represent it using a volume of voxels.

Each voxel could encode information through:

Position (X, Y, Z)

Color

Opacity

Density

Texture

Material

Connections to neighboring voxels

You now have considerably more encoding possibilities.

2. Imagine a multidimensional addressing system
Suppose we have a semantic embedding for an object.

A high-dimensional vector might look something like:

[0.82, 0.71, 0.15, 0.43, 0.92, ...]
You could project that vector into a three-dimensional voxel grid.

Volumetric Heatmap - Questions - Babylon.js
For example:

Voxel property

Encoded information

X position

Semantic dimension 1

Y position

Semantic dimension 2

Z position

Semantic dimension 3

Color

Semantic cluster

Opacity

Confidence

Density

Information intensity

This would create a spatial representation of information.

3. You could encode entire objects as voxel structures
Imagine representing a hot dog and a hamburger.

Hot dog

One semantic voxel structure

Smart Lens Cursor: Free Interactions Component by Muhmad Haj Hmdo — Framer Marketplace
Hamburger

A neighboring semantic structure

Their structures might overlap significantly in a shared semantic space.

A moon would occupy a substantially different region.

Importantly, there are two different approaches here:

Voxelized embedding: Voxels represent positions in a semantic space.

Voxelized object: Voxels represent the actual structure or properties of the object.

You could use both, but they solve different problems.

4. Here's where I think your original identification concept becomes powerful
Imagine a system with several addressing layers:

Multidimensional information addressing
Layer 1 — Identity

Unique resource identifier.

Layer 2 — Semantic address

High-dimensional numerical vector.

Layer 3 — Spatial encoding

Voxel coordinates, occupancy and density.

Layer 4 — Chromatic encoding

Perceptually meaningful color assignments.

Layer 5 — Relationships

Connections, proximity, overlap and neighborhoods.

5. Some existing technologies worth investigating
These are particularly relevant:

Sparse Voxel Octrees — Efficiently represent large three-dimensional spaces without storing every empty voxel.

OpenVDB — An open-source library for sparse volumetric data structures.

Morton Encoding — Converts multidimensional coordinates into numerical sequences.

Hilbert Curves — Maps multidimensional spatial positions into ordered sequences.

Vector Quantization — Compresses high-dimensional data into discrete numerical representations.

These could provide building blocks for your architecture.

6. One question that fundamentally changes the design
When you say you want to encode underlying data into voxels, I see two potentially very different ambitions:

Which of these is closer to what you're imagining?


A. A universal information space
Selected

About Lytics | Lytics Lab
Every resource gets a position in a shared three-dimensional universe. Related things occupy nearby regions. The voxels represent semantic relationships.

B. A universal data structure

NTKs & Sparse Data: Unlocking Potential In Rare Domains
Every resource is encoded as a voxel structure. Its geometry, colors and internal arrangement represent its underlying data.
My suspicion is that what you're ultimately describing could combine both: a shared spatial addressing system in which individual resources have their own volumetric representations.

Yeah, more or less I wanted to see if I could take these encoded voxel structures or and then turn essentially populate or seed I guess Minecraft worlds are seeded. What I would like to do sort of provide a construct space like a place where people can store their data in a way that populates a shared world. And then use those primitives as the building blocks for procedurally generated
Oh, now I think I see the bigger idea.

You're describing something like a procedurally generated, shared world whose geography and architecture emerge from the information people contribute to it.

Not just a Minecraft-like visualization of files. More like a world that is constructed from data.

People contribute information, identities, relationships, and other resources. Those are encoded into spatial primitives. Those primitives populate a shared world, and that world becomes a navigable, extensible environment.

I particularly like your use of the word seed. It suggests something important: the world doesn't necessarily need to store every visual detail. It can store the underlying information and deterministic rules needed to generate the environment.

1. Think of it as a data-driven world generator
Home | Starfish Studios
Consider the following pipeline:

01. Contributions

Documents, projects, code, images, datasets, agents, people, relationships.

02. Encoding

Assign identities, extract metadata, generate embeddings, establish relationships.

03. Spatial primitives

Convert information into voxel structures, coordinates, materials and connections.

04. World generation

Generate terrain, buildings, biomes, regions, pathways and structures.

05. Shared world

People explore, contribute, modify, discover and build upon the world.

The interesting thing is that the world becomes an interface to the information, rather than the information being something you put inside an existing world.

2. The world could have an underlying mathematical geography
I'd distinguish between three types of space.

Semantic space: Where things belong conceptually.

For example, programming languages might cluster together, while recipes occupy another region.

Relational space: Where things belong according to their actual connections.

A project, its contributors, its repositories and its documentation might form a connected neighborhood.

Physical space: Where things are rendered in the generated world.

These three spaces could influence one another, but shouldn't be identical.

For example, two programming languages might be semantically close but physically separated because they belong to different people's territories.

That distinction gives you tremendous design flexibility.

3. Information becomes building material
This is where I'd start designing the actual primitive vocabulary.

Rather than treating every document as a block, you could define a small grammar of spatial objects.

Chunk Survival — Multiplayer Voxel Territory Survival
Resource blocks

Documents, files, datasets, media and other individual resources.

Chunk Survival — Multiplayer Voxel Territory Survival
Relationship structures

Links, dependencies, references and associations.

Starfish Studios | Official Minecraft Partner
Territories

Projects, organizations, domains and collections.

Blockwave Studios (@BlockwavesTeam) on X
Structures

Applications, agents, workflows and composite systems.

I'd also introduce:

Materials: Represent resource types, provenance, or other properties.

Biomes: Represent broad semantic or organizational environments.

Landmarks: Important resources or particularly connected entities.

Portals: Navigation between otherwise distant regions.

Terrain: The underlying distribution of information density.

The world becomes a visual grammar for information.

4. Minecraft-style seeding, but with actual data
This is probably the most important technical distinction.

Minecraft can generate a world from a seed because its generation algorithms are deterministic.

You could use a similar approach, but your seed would have more components.

I'd propose:

world:
  id: "world-001"

  seed:
    base: "0x8FA31C"
    data_snapshot: "sha256:..."
    generator_version: "1.0"
    spatial_schema: "1.0"

  generation:
    semantic_projection: "embedding-v1"
    terrain: "relational-density"
    materials: "chromatic-addressing"
    structures: "resource-grammar"

  chunks:
    size: 32
    addressing: "morton"

The generator would take the seed, the data snapshot and the rules, then deterministically produce the world.

This has some significant advantages:

You don't need to store every generated voxel.

Different users can independently reconstruct the same world.

You can regenerate chunks when necessary.

You can version the world-generation algorithms.

You can share worlds without distributing enormous world files.

One subtlety: changing the entire seed whenever someone contributes new data could regenerate everything. I'd instead use stable spatial addresses and incremental chunk generation.

5. The shared world needs a distinction between generated and constructed space
I'd make two different types of territory.

Generated territory

UPDATE 0.9.0 IS AVAILABLE RIGHT NOW! · Colony Survival update for 22 December 2022 · SteamDB
Generated from underlying data and rules. Its geometry reflects the information it represents.
Constructed territory

Minecraft Portfolio — Antares BT
Created or modified by people and agents. Users can build structures, organize resources, add annotations and create new connections.
This gives you a really interesting feedback loop:

Data generates a world.

People explore the world.

People construct new things.

Those constructions generate new data.

The world evolves.

You'd need provenance and permissions so that manually constructed objects aren't accidentally overwritten by regeneration.

6. Existing technologies I'd investigate
You don't necessarily need to build a voxel engine yourself.

Technology

Potential role

Luanti

Open-source voxel world engine

Minetest Voxel Manipulators

Programmatic world modification

OpenVDB

Sparse volumetric data storage

Godot

Custom interactive world client

Bevy

Potential Rust-based rendering engine

H3

Hierarchical spatial addressing

I'd particularly investigate Luanti as an initial prototype because you could experiment with an existing voxel world rather than first building rendering, chunk management, and multiplayer infrastructure.

7. The architecture I'd aim for
I would separate this into independently composable systems:

Data-native world architecture
1

Data layer

Resources, files, databases, contributions

2

Identity layer

Johnny Decimal, UUIDs, semantic addresses, namespaces

3

Spatial layer

Vectors, voxel coordinates, spatial indexing

4

Material layer

Colors, textures, properties, visual identity

5

Generation layer

Terrain, structures, biomes, procedural rules

6

World layer

Chunks, persistence, multiplayer, permissions

7

Interaction layer

Exploration, construction, agents, navigation

8. The idea I'd be most excited to experiment with
Imagine that every person, organization, or agent has a way of contributing to this shared world.

A software project might generate a mountainous region:

Repositories become structures.

Dependencies become bridges.

Documentation becomes libraries.

Issues become points of interest.

Contributors become inhabitants.

Related projects become neighboring territories.

A research organization might generate an entirely different kind of environment.

A personal knowledge base might generate a landscape where concepts, notes and relationships determine the geography.

And because everything is derived from an underlying addressing system, you can navigate the world in both directions:

Information → World: Find where information lives in the generated environment.

World → Information: Discover the underlying resources by exploring the environment.

The fundamental design decision
There is one thing I'd be careful about before committing to an implementation.

I'd distinguish between:

A. The world as a visualization of data. The world is essentially a 3D interface to a database.

B. The world as a persistent computational environment. The world is itself a place where data, agents, people, applications and workflows exist and interact.

I'd lean strongly toward B.

That would make this less like a 3D file browser and more like a programmable, data-native metaverse: a shared environment whose geography is derived from information, but whose inhabitants can actually do things within it.

And crucially, the world-generation engine, addressing conventions, voxel primitives and data sources could all be independently replaceable.

This might also interest you.



If I was going to use something like Gordian envelopes  let's say I had I want to, like there's a data structure or there's this data that I want to encrypt. But I want to have different layers of this dimensionality that we're exploring provided in plain text. Would something like a Gordian envelope allow me to keep a raw data like anywhere? And also allow for this sort of persistent world
Yes. And I think Gordian Envelopes could be a particularly good fit for this architecture, because they address something you've been circling around: separating the structure and metadata of information from the information itself, while allowing different parts to have different privacy and disclosure properties.

The idea I'd explore is:

An encrypted information object that carries a publicly inspectable description of its identity, spatial position, relationships, and rendering instructions, while keeping its actual payload private.

That could become the fundamental building block of your persistent world.

There is one important distinction: Gordian Envelopes can provide the structured, cryptographically protected information container, but they aren't themselves a spatial database, voxel engine, or persistent world engine. We'd build those capabilities around them.

1. Separate the envelope into information layers
Imagine contributing a document to your world.

You want the world generator to know:

What kind of resource it is.

Where it belongs conceptually.

Its spatial coordinates.

Its visual representation.

What other resources it relates to.

Who owns it.

Whether it is publicly accessible.

But you don't necessarily want the world generator to know the document's actual contents.

I'd model that as several layers:

Conceptual information envelope
Public metadata

Plaintext
Identity, resource type, schema version, public ownership information.
Spatial metadata

Plaintext
Semantic addresses, voxel coordinates, color, relationships and world location.
Access metadata

Plaintext / selectively disclosed
Encryption information, recipient information, access policies and key references.
Original payload

Encrypted
Original files, documents, images, datasets, private content.
The world engine could read the spatial metadata and construct the appropriate voxel structures without decrypting the original payload.

2. Gordian's recursive structure is especially useful here
One of the features I particularly like is that an envelope can contain other envelopes.

You don't have to make the entire object public or the entire object private. You can construct nested structures with different disclosure boundaries.

For example, conceptually:

RESOURCE ENVELOPE
│
├── Identity
│   ├── Resource ID
│   └── Identification scheme
│
├── Public spatial assertions
│   ├── Semantic address
│   ├── Voxel position
│   ├── Color coordinates
│   ├── World / region
│   └── Public relationships
│
├── Ownership and permissions
│   ├── Owner identity
│   ├── Access policy
│   └── Recipient permits
│
└── Encrypted nested envelope
    ├── Original document
    ├── Private metadata
    ├── Original semantic embedding
    └── Private relationships
This isn't just a theoretical fit.

Gordian Envelope supports recursive structures, cryptographic commitments, encryption, signatures and selective disclosure through hashed elision. 

Developer Resources
+2

There is an especially important technical detail: encrypting an envelope's subject doesn't automatically encrypt all its assertions. If you want the entire inner structure protected, you need to wrap the structure before encrypting it. 

GitHub
+1

That distinction matters enormously for your design.

3. The world can operate without opening the envelope
Imagine a person contributes a confidential research document.

The world engine receives something like:

resource:
  id: "resource-123"

  public:
    type: "research-document"
    semantic_address: "12.04.08.03"

    spatial:
      region: "world-001/region-042"
      coordinates: [120, 45, 18]

    appearance:
      color: "#D97736"
      primitive: "document-crystal"

    relationships:
      - resource-456
      - resource-789

  private:
    payload:
      envelope: "encrypted"
      storage_reference: "content-address"

The world generator can:

Recognize the resource.

Place its representation in the world.

Render its color and geometry.

Connect it to neighboring resources.

Show its public relationships.

Leave its contents encrypted.

Someone walking through the world can discover the resource without being entitled to read it.

Someone with the appropriate cryptographic permit can open it.

Gordian's multiple-permit model is particularly relevant here: different people can have different cryptographic ways of opening the same protected content. 

Developer Resources
+1

4. The really interesting part: the envelope becomes a persistent world object
I'd give every world object a stable cryptographic identity.

Rather than storing the entire rendered object in the world database, store its underlying description.

For example:

Persistent resource object
Resource

Persistent identity
Cryptographic identity

Envelope digest / content identifier

sha256:8f31...

World placement

world-001 / region-042 / chunk-120

Voxel representation

Geometry, color, material and relationships.

Payload reference

External encrypted data storage.

The resource's identity and its rendered appearance are separate. You can change the renderer without changing the underlying resource.

This suggests an architecture where:

Envelopes are the authoritative information objects.

A spatial index records their world locations.

A procedural engine generates their visual representations.

A persistent event log records contributions and world modifications.

Object storage holds the actual encrypted payloads.

I would not make the voxel world itself the source of truth.

The world should be a reproducible projection of the underlying information and its history.

5. Can the original data live anywhere?
Yes. I would deliberately avoid coupling the envelope format to a particular storage provider.

You could store encrypted payloads in:

Storage backend

Potential purpose

Local filesystem

Personal ownership and offline access

S3-compatible storage

Large files and scalable hosting

IPFS

Content-addressed distribution

Arweave

Long-term archival

Git repositories

Versioned text and code

Private servers

Organizational storage

The envelope can contain the content itself or references and metadata describing where content can be retrieved. Its cryptographic structure gives you a way to verify retrieved content independently of the storage provider. 

GitHub
+1

I would use content-addressed storage for the actual payloads and keep the envelope as the portable, verifiable description.

One caveat: public metadata is genuinely public. Spatial coordinates, relationships, ownership and access patterns can reveal sensitive information even when the payload is encrypted. I'd make each metadata field independently configurable for disclosure.

6. An additional idea: the world itself could be a collection of envelopes
This is where I think you could go beyond a conventional game architecture.

What if these were all envelopes?

World envelope

Region envelope

Chunk envelope

Structure envelope

Resource envelope

Relationship envelope

User/agent identity envelope

A region could contain references to chunks. Chunks could contain references to structures. Structures could contain references to resources.

Each object could have its own identity, signatures, permissions and revision history.

This gives you a recursively composable world.

Chunk Survival — Multiplayer Voxel Territory Survival
I'd also investigate Blockchain Commons' Gordian Clubs work. Its architecture explores shared control, permissions and evolving editions of encrypted documents, which could be relevant to collaborative world ownership. 

HackMD
+1

7. My proposed technology architecture
Layer

Technology / approach

Cryptographic objects

Gordian Envelope

Binary representation

Deterministic CBOR

Identity

Content digests and independent namespaces

Payload storage

Pluggable object storage

Relationships

Envelope assertions / graph

Spatial addressing

Morton or Hilbert encoding

Voxel representation

Sparse voxel structures

World generation

Deterministic procedural generator

Persistence

Append-only event history

Rendering

Luanti initially, custom engine eventually

8. The biggest architectural opportunity
I would make one principle fundamental:

The world should be generated from cryptographically verifiable information, not be the place where the information is stored.

That means you can:

Rebuild the world from its underlying data.

Move the data between storage providers.

Change the voxel renderer.

Introduce new identification schemes.

Add new spatial dimensions.

Share only selected portions of information.

Give agents access to world metadata without giving them access to private documents.

Allow people to contribute to a shared world without surrendering ownership of their underlying data.

And you could eventually have multiple different worlds generated from the same underlying collection of envelopes, each with different spatial projections and generation rules.

One particularly exciting possibility: the same encrypted resource could appear in multiple worlds, in different locations, with different visual representations, without duplicating or changing the original resource.

That's the direction I'd pursue: Gordian Envelopes as the portable, cryptographically verifiable information substrate, with a persistent procedural world as one of its many possible projections.

I think it'd be interesting to have a sort of decentralized network storage network. It doesn't have to be all, like all data on Network could be Storage doesn't have to be on the network. It could be the verifiable information pointers Okay metadata anyway
Yes — and I think that distinction is important. You're describing something closer to a decentralized network of verifiable information and storage references, rather than a decentralized filesystem where everybody is expected to store everything.

The network doesn't need to possess the data itself. It needs to be able to describe:

What exists — a resource with a stable identity.

What it claims to be — metadata, classification, semantic and relational information.

Where it can be found — one or more storage pointers or retrieval methods.

How to verify it — cryptographic commitments, signatures, and integrity checks.

Who can access it — permissions, keys, and disclosure rules.

The actual payload might live on someone's laptop, a private server, an S3 bucket, IPFS, or several places at once. The network can still make its identity and published claims verifiable.

In other words: decentralize the description, identity, discovery, and verification of information — not necessarily the storage of every byte.

I'd give this its own architectural layer. Something like a Verifiable Metadata Network — a decentralized fabric that connects information, regardless of where the underlying information is stored.

1. Separate the network into four layers
Layer 1
Identity and addressing

Stable resource IDs, namespaces, identification schemes, version identifiers and cryptographic fingerprints.

Layer 2
Metadata and relationships

Descriptions, semantic classifications, vector representations, relationships, ownership claims, provenance and permissions.

Layer 3
Discovery and verification

Distributed indexes, signed records, cryptographic commitments, resource resolution and availability announcements.

Layer 4
Storage and retrieval

Local disks, private servers, cloud storage, IPFS nodes, archives, or other networks.

The important thing is that these layers don't have to be operated by the same people, organizations, or infrastructure.

2. What would actually be decentralized?
Imagine I create a resource:

Resource: Architectural design document

Illustrative record

Property

Value

Identity

resource:abc123

Type

Architectural document

Semantic location

Architecture → Buildings

Relationships

Project X, Building Y

Integrity

SHA-256 commitment

Owner

Public-key identity

Storage

Multiple locations

Disclosure

Public metadata, private payload

Storage references

file:///home/user/project/design.pdf

s3://private-bucket/design.pdf

ipfs://bafy...

https://example.org/design.pdf

The network can distribute and verify the metadata record without distributing the document.

If someone retrieves the document from any of those locations, they can check whether its contents match the expected cryptographic commitment.

This is already close to the architectural model used by IPFS: content identifiers are independent of storage locations, while distributed hash tables help discover peers providing content. 

IPFS Docs
+1

3. The distinction I'd preserve: identity, pointer, and proof
These are three different things:

Concept

What it answers

Identity

Which resource are we talking about?

Pointer

Where might I retrieve it?

Proof

How do I know what I retrieved is authentic?

A pointer is not proof that a resource is still available. A cryptographic hash is not proof that the resource is trustworthy or that its description is true.

You need all three, independently.

I'd also make pointers multi-valued. One resource could have several storage providers, each advertising availability, access requirements and perhaps a freshness timestamp.

For mutable resources, a stable identity can resolve to a newer signed version. IPNS is one existing example of this pattern: a mutable, verifiable name points to changing content identifiers. 

IPFS Docs

4. Where Gordian Envelopes fit
This is where your earlier idea becomes particularly interesting.

A Gordian Envelope could represent the resource's structured, cryptographically verifiable metadata:

Public identity and descriptive assertions.

Public or selectively disclosed semantic coordinates.

Relationships to other resources.

Storage pointers.

Cryptographic commitments.

Encrypted nested metadata and payload references.

Signatures and provenance.

Different parts could be independently disclosed or protected. The actual payload doesn't need to be inside the envelope.

One envelope could describe a resource that exists in several different storage systems, and several different applications could interpret that same envelope.

5. How this connects to your persistent world
This gives the world a much more interesting foundation.

The world doesn't need to contain the files. It can be a spatial projection of the network's verifiable metadata.

Local-First Software for SaaS Builders: Honest Guide 2026
Chunk Survival — Multiplayer Voxel Territory Survival
How to set up Multiple Minecraft Servers on 1 IP ➡ Blog BlueServers.com
For example:

A document becomes a structure.

A project becomes a territory.

A relationship becomes a path or bridge.

A collection becomes a settlement.

A semantic cluster becomes a biome.

An agent becomes an entity that can navigate, inspect and interact with resources.

And because the world is derived from metadata, it can be regenerated, explored through different visualizations, or accessed by agents without requiring every agent to download every payload.

There is one important privacy consideration: public metadata can itself reveal sensitive relationships, interests, ownership and activity. IPFS explicitly warns that provider records and content identifiers can be publicly observable. 

IPFS Docs

6. The architecture I'd investigate
I would not build this as one monolithic decentralized storage protocol. I'd compose existing technologies:

Responsibility

Technology to investigate

Structured cryptographic records

Gordian Envelopes

Content-addressed objects

IPFS / IPLD

Distributed discovery

Kademlia DHT

Mutable signed references

IPNS or equivalent

Peer identity and transport

libp2p

Local and private storage

Filesystems, S3-compatible stores

Semantic discovery

Vector indexes and graph indexes

Persistent world

Voxel engine + event log

IPFS itself documents an architecture where content addressing, routing, and data transfer are separate subsystems. It also describes patterns where files remain in original storage while a separate index tracks their content identifiers and metadata. 

IPFS Docs
+1

One design decision I think is particularly important: don't make the decentralized network's global index the authoritative source of truth. Let independently signed resource records be authoritative; let network indexes be replaceable discovery mechanisms.

That way, if one indexer disappears, another can rebuild its index from records. If a storage provider disappears, another pointer can still work. If the world renderer changes, the underlying information remains intact.

The bigger idea is that you're not really building a distributed hard drive. You're building a decentralized information-addressing and verification layer, with storage providers as interchangeable backends and the persistent world as one possible interface to that information.

That's a much more flexible foundation.

This might also interest you.



