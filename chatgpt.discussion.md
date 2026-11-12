Besides voxels, is there any other encoding that I could populate a 3D world with
Absolutely. In fact, I think voxels are only one of several possible ways to turn information into a populated 3D world. They are particularly good for Minecraft-like construction, but you could build something much more expressive by combining several representations.

The key distinction is that you don't have to encode information as a physical block. You can encode it as geometry, topology, fields, particles, structures, or even rules that generate geometry.

I'd think about this as a 3D world representation system, with multiple interchangeable spatial primitives.

1. Different ways to represent information in 3D
Modern building downtown

1. Polygon meshes

Objects represented by vertices, edges and faces.

Think: buildings, furniture, machines, monuments, terrain.

A project could become an architectural structure, with its individual documents represented by rooms or components.

Best for: editable objects and recognizable geometry.

2. Point clouds

A collection of 3D coordinates, optionally carrying color, labels, weights and other attributes.

Each point could represent a resource, concept, person, event or data record.

Best for: sparse, large-scale information landscapes.

Automated Theorem Proving for AI Safety: Proving Alignment Preservation Under Self-Modification

3. Graphs and networks

Nodes represent entities; edges represent relationships. The graph can be embedded into 3D coordinates and rendered as a navigable structure.

A connection between two projects could literally become a bridge. A dependency could become a pipeline.

Best for: relationships, knowledge graphs, dependencies and social structures.

4. Scalar and vector fields

Instead of defining individual objects, define a value or vector at every position in space.

For example, semantic density could create hills; information activity could create currents; similarity could create valleys that attract related resources.

Best for: continuous semantic landscapes and emergent environments.

Glowing particles  | SideFX

5. Particles and Gaussian splats

Represent objects or information as spatially distributed particles, each with properties such as position, size, orientation, color and opacity.

Gaussian splatting uses oriented 3D Gaussian distributions to produce smooth, volumetric-looking visualizations.

Best for: fluid, organic, dense or animated representations.

Isopod - implicit surface tools - Page 5 - Grasshopper - McNeel Forum

6. Implicit surfaces and signed distance fields

Rather than storing the surface itself, store a mathematical function that determines where the surface exists.

This could generate organic terrain, caves, boundaries and structures that smoothly merge together.

Best for: procedural geometry and continuous shapes.

7. Procedural grammars and parametric structures

Store rules and parameters instead of finished geometry.

For example: a resource collection generates a library; a library generates rooms; rooms generate shelves; shelves contain documents.

Best for: infinitely extensible, data-driven worlds.

3D Gaussian Splatting : nouvel outil favori des artistes numériques ? - Fisheye Immersive

8. Neural fields and radiance fields

A learned function represents a scene's appearance or geometry. NeRF, for example, models density and color as a function of spatial position and viewing direction.

Best for: rich visual reconstruction and learned representations, though less suitable as the primary editable data structure.

These are established families of 3D scene representations, with different trade-offs in editability, resolution, memory and rendering. 

IJCAI
+3

2. The more interesting possibility: combine them
I wouldn't choose one representation for the entire world.

Imagine a world in which:

The graph determines what is connected.

The semantic coordinates determine conceptual proximity.

The point cloud determines where individual resources are distributed.

The field determines the surrounding landscape.

The procedural grammar determines how structures are built.

The mesh provides the physical architecture.

The voxels provide editable, block-based construction where appropriate.

Going Meta S02E04 – a Series on Semantics, Knowledge Graphs and All Things AI - Conferences, Meetups, & Events - Neo4j Online Community

Scientists Have Discovered a Link Between Finance and Topology

IT Communication: Translating Tech for Business | IT Support Group

This is a hybrid representation. Different representations can describe different aspects of the same underlying resource.

For instance, a project might have:

Representation

What it expresses

Graph

Its relationships to other projects

Point cloud

Its position in semantic space

Field

Its influence on nearby regions

Procedural grammar

How its territory is constructed

Mesh

Its buildings and structures

Voxel overlay

User modifications

3. One idea I'd particularly explore: information as a generative grammar
This might be the most interesting direction for your project.

Instead of encoding a document as a block, encode it as a rule-bearing entity.

For example:

Project
  ├── generates a territory
  ├── contains collections
  │     ├── generate buildings
  │     └── contain resources
  ├── has dependencies
  │     └── generate bridges
  └── has semantic coordinates
        └── influence landscape placement
The resource doesn't need to know how to draw itself. It exposes its identity, properties, relationships and perhaps a rendering profile. The world engine interprets those properties.

This means different worlds could render the same underlying data differently.

One world could look like a fantasy landscape. Another could look like a futuristic city. Another could be an abstract network visualization.

4. A distinction that could shape the entire architecture
I'd separate three things:

A. Information representation

What the data actually means: identity, metadata, semantics, relationships and provenance.

B. Spatial representation

How that information is encoded in 3D: points, graphs, fields, meshes, voxels, grammars.

C. World presentation

How a particular environment renders and allows interaction with those representations.

This would let you build a system where the data is independent of the world engine.

My recommendation: make the underlying information graph and its address spaces authoritative, then make 3D representations pluggable. The world is an interpretation of the information, not the information itself.

One particularly interesting next step would be to define a small universal set of spatial primitives that can be composed into all these representations. That could become the equivalent of a graphics-independent language for your world.

For securing decentralized storage and identity, here’s a PQC encryption risk option.



i mean, is there something like a voxel with more dimensionality?
Ah, yes. You're asking about a higher-dimensional equivalent of a voxel: a discrete spatial cell that can carry information, but exists in more than three dimensions.

The term you're looking for is hypervoxel.

1. The dimensional progression
Canvas Widget（キャンバス･ウィジェット） | TouchGFX Documentation 4.21

2D — Pixel

A cell in a two-dimensional grid.

Ch. 6 Voxels | Matplotlib with Rmarkdown

3D — Voxel

A cell in a three-dimensional grid.

Tesseract - Polytope Wiki

4D — Tesseract / hypervoxel

A cell in a four-dimensional grid.

Penteract — 5D Rotating Cube of Cubes by blackwall on Dribbble

5D+ — Hypercube cells

Cells in higher-dimensional grids.

A tesseract is the four-dimensional analogue of a cube. Its 4D volume element is a hypercube, also called a 4-cube.

The terminology is slightly nuanced: hypervoxel is the generalization; tesseract specifically describes a 4D hypercube. Some literature also proposes rexel for a four-dimensional hypervoxel. 

Wiley Online Library
+1

2. Imagine giving every dimension a meaning
This is where it gets interesting for your project.

Instead of a voxel with just X, Y and Z, imagine a cell with coordinates:

(
x
,
y
,
z
,
s
,
t
,
p
,
a
)
(x,y,z,s,t,p,a)
For example:

Dimension

Meaning

X, Y, Z

Physical position

S

Semantic position

T

Time / historical state

P

Provenance / source

A

Access or authorization domain

A single hypervoxel could represent a particular resource in a particular semantic region, at a particular point in its history, with particular provenance and access characteristics.

You could have dozens or hundreds of dimensions, in principle.

3. The really interesting part: render a projection into 3D
You don't actually need to display all dimensions simultaneously.

You could have a seven-dimensional information space, but render different three-dimensional projections of it.

可视化教程 - Meta Theory of the Zeckendorf-Hilbert Universe

For example:

Physical view: X, Y, Z.

Semantic view: semantic dimensions projected into X, Y, Z.

Historical view: time becomes a spatial axis.

Provenance view: sources become separate layers or regions.

Access view: permissions determine which dimensions or cells are visible.

You could even let someone navigate between these projections.

4. A concept worth investigating: multidimensional arrays
There is an important distinction between a hypervoxel as a mathematical cell and the data structure used to store it.

A multidimensional array (N-dimensional tensor) can store values over an arbitrary number of axes. Sparse N-dimensional grids can avoid storing empty cells.

Some useful concepts:

N-dimensional arrays / tensors: general-purpose multidimensional data.

Hypervoxel grids: discrete cells in N-dimensional space.

N-dimensional sparse trees: hierarchical storage of occupied regions.

Hilbert curves: mappings between multidimensional grids and linear addresses.

Cell complexes: more general topological structures than regular grids.

For example, sparse hierarchical grids and their higher-dimensional generalizations are active areas of research. 

Ken Arroyo Ohori
+1

My thought: what you're describing could be a kind of multidimensional information substrate.

The 3D world is just one projection of that substrate. The underlying cells can encode semantic, temporal, relational, cryptographic and other dimensions.

One question that would significantly shape the design: Do you imagine these extra dimensions being actual independent axes of the data space, or additional properties carried by each cell?

Those are two very different architectures, and you might ultimately want both.

This might also interest you.



The only thing I was thinking is that Minecraft looks like shit kind of and blockiness, and it would be cool if there was like higher resolution something like that
Ahhh, now I understand. You're not necessarily looking for more mathematical dimensions. You're looking for the equivalent of a voxel, but with much finer resolution and smoother, more sophisticated geometry.

You want the flexibility of Minecraft's world-building, without everything looking like Lego.

There are several technologies that get much closer to that.

The main alternatives
Check Out This Unity-Powered Voxel Manipulation Sandbox Game Demo

1. Smooth voxels (Marching Cubes / Dual Contouring)

Still uses a 3D grid, but extracts smooth surfaces from it. You can get rolling hills, organic architecture, caves and smooth transitions.

Closest upgrade to Minecraft
Cloud Layer Puffy Cumulus VDB | Clouds  models | Blendkit

2. Volumetric fields (OpenVDB)

Instead of solid cubes, space is represented by sampled values such as density, distance or material. Sparse storage makes very large volumes practical.

Great for organic structures, clouds, terrain and complex procedural shapes.

fantasy architecture set w2r_28 | Fab

3. Polygon meshes

Arbitrarily detailed surfaces made from triangles or other polygons. You aren't restricted to a fixed grid resolution.

Great for buildings, objects, landmarks and detailed architecture.

Il 4D Gaussian Splatting -           Intro Gaussian Splatting in V-Ray 7

4. Gaussian splats

Smooth, overlapping 3D Gaussian primitives. They can create highly detailed, almost photographic scenes.

Particularly interesting if you want an information landscape to have an organic, atmospheric appearance.

Minimal Desk

5. Signed Distance Fields (SDFs)

Store a mathematical distance to a surface rather than individual blocks. This makes smooth shapes, blending and Boolean geometry operations possible.

Particularly interesting for procedural world generation.

What I'd investigate for your world
I would look seriously at a combination of three technologies:

A smooth, data-driven world
Glass Dome Sanctuary in a Verdant Future Valley - backiee

OpenVDB / sparse volumetric fields

For the underlying terrain and volumetric structures.

SDFs and procedural geometry

For generating smooth structures, buildings and organic transitions.

Polygon meshes

For rendering detailed objects and architecture efficiently.

The really important thing is that you can retain the logic of voxels without retaining their appearance.

You can still have discrete, addressable cells representing information. But instead of rendering each cell as a cube, you can render its boundaries as smooth surfaces, blend neighboring cells, or generate geometry from their combined properties.

That gives you a potentially enormous information space that feels like an actual world rather than a giant Minecraft map.

I'd particularly look at OpenVDB and Dual Contouring. They're good starting points for exploring this direction.

1
Smooth voxels — that's a good direction.

The idea is to keep the voxel's underlying advantages (discrete, addressable, composable cells) but stop rendering them as cubes.

Instead, you use a 3D grid as the underlying data structure and generate a smooth surface from the values stored in that grid.

The two techniques I'd investigate first are Marching Cubes and Dual Contouring.

A visual guide to voxels

Marching Cubes

Samples a density field and creates a smooth triangular surface where it crosses a threshold. Excellent for natural terrain.

Ultimate Terrains - Voxel Terrain Engine

Dual Contouring

Generates surfaces while preserving sharper corners and edges. Better suited to mixing natural landscapes with human-made structures.

1. The underlying trick: store density, not solid blocks
Instead of a cell being either solid or empty:

1 = solid
0 = empty
Store a continuous value:

-1.0 = inside a surface
 0.0 = exactly on the surface
+1.0 = outside the surface
This is a signed distance field (SDF). It lets you describe smooth shapes mathematically, then extract their surfaces. 

Voxel Tools documentation
+1

2. You can make resolution adaptive
This is particularly important for your proposed world.

You don't need every region to have the same resolution.

Nearby structures: very high detail.

Distant terrain: lower detail.

Important information landmarks: exceptionally high detail.

Empty regions: almost no stored data.

This is achieved with hierarchical spatial structures such as octrees and level-of-detail systems. Transvoxel is one technique for joining terrain chunks with different resolutions without visible cracks. 

transvoxel.org
+1

3. What I'd prototype
My recommendation SDF + Dual Contouring + adaptive octree

Component

Responsibility

SDF

Describes the shape

Octree

Stores spatial data efficiently

Dual Contouring

Generates smooth geometry

LOD

Adjusts detail based on viewing distance

Material system

Gives regions their visual identity

A useful implementation reference is Voxel Tools. Its documentation explains smooth SDF terrain and Transvoxel meshing: Smooth terrain documentation.

And here's the important architectural point: your information objects could still have stable discrete addresses, even if their rendered geometry is completely smooth.

You get the addressability of Minecraft, without the Minecraft aesthetic.
