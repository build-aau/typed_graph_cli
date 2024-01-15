# migration_handler
Introduces BUILDscript a declarative language for describing the schemas of a graph and Changeset a declarative language for keeping track of changes between schemas

[![Crates.io](https://img.shields.io/crates/v/migration_handler.svg)](https://crates.io/crates/migration_handler)
[![docs.rs](https://img.shields.io/docsrs/migration_handler)](https://docs.rs/migration_handler/latest/migration_handler/)

## Getting started
### Creating a project
migration_handler is mainly controled using a command line interface. However all functionlities in the cli is also throught the cli module.

To create a new project run
```
migration_handler new
```
This creates two folders schemas and changesets.
the schemas folder contains all the BUILDscript files with the extension *.bs
the changesets folder contains all the Changesets with exstension *.bs.diff

### Generating code
if we have the schema
```
<V0.0>
node A{};
node B{};

edge AB(A => B){};
edge ABSingle(A =>[n <= 1] B){};
```
and want to use it in typed_graph. We export it using the export rust function.
The export function need the path to where it should generate the files.
So given a project like this:

```
|-- schemas
|-- changesets
|-- src
|   |-- lib.rs
|-- Cargo.toml
```

running export rust
```
migration_handler export rust src/graph
```

Will produce a bunch of files that can be imported like any other model in rust
> export rust will only add new files never delete old ones.
```
|-- schemas
|-- changesets
|-- src
|   |-- graph
|   |   |-- v0_0
|   |   |   |-- nodes           <-- Folder containing all node types
|   |   |   |   |--a.rs         <-- Implentation of node A
|   |   |   |   |--b.rs         <-- Implementation of node B
|   |   |   |   |--mod.rs
|   |   |   |-- edges
|   |   |   |   |--ab.rs        <-- Implementation of edge AB
|   |   |   |   |--ab_single.rs <-- Implementation of edge ABSingle
|   |   |   |   |--mod.rs
|   |   |   |-- structs         <-- Folder containing all struct type
|   |   |   |-- type            <-- Folder containing all enum types
|   |   |   |-- schema.rs       <-- Implementation of V0_0 schema
|   |   |   |-- nodes.rs        <-- Joined enum of contianing all nodes
|   |   |   |-- edges.rs        <-- Joined enum of contianing all edges
|   |   |   |-- node_type.rs    <-- Joined enum of contianing all nodes without data
|   |   |   |-- edge_type.rs    <-- Joined enum of contianing all edges without data
|   |   |   |-- imports.rs      <-- Place to insert imports specific for V0.0
|   |   |   |-- mod.rs
|   |   |-- imports.rs          <-- Place to insert global imports
|   |   |-- mod.rs
|   |-- bin.rs
|-- Cargo.toml
```
each schema creates their own module with a complete schema implementation that can be used directly in typed_graph

bin.rs
```rust
mod graph;

use graph::v0_0::*;

type NodeKey = usize;
type EdgeKey = usize;

fn main() -> SchemaResult<(), KeyType, KeyType, V0_0<KeyType, KeyType>> {
    let mut g = TypedGraph::<NodeKey, EdgeKey, V0_0<NodeKey, EdgeKey>>::default();
    g.add_node(A::new(0))?;
    g.add_node(B::new(1))?;

    g.add_edge(0, 1, AB::new(0))?;
    g.add_edge(0, 1, ABSingle::new(1))?;

    // This will fail with InvalidEdgeType - ToMany
    // g.add_edge(0, 1, ABSingle::new(2))?;

    // This will fail with InvalidEdgeType - InvalidType
    // g.add_edge(1, 0, AB::new(2))?;
    Ok(())
}
```
### Working with changesets
a changeset between two schemas can be made using 

```
migration_handler migration link V0.0 V0.1
```
Where V0.0 and V0.1 is the name of the schema in the top of the schema file.

Once a changeset is made the program will fail if any of the two schemas changes. 

> When updating the program changesets might become invalid.
> To handle this run migration update
```
migration_handler migration update
```

Removing a changeset is done by deleting the corresponding file in the changesets folder.

The changesets must form a tree. Means cycles are not allowed.
This makes it easy to use migrations as you are always guaranteed that if a schema is part of the changeset tree there is only one way to migrate to a newer version.

The side effect is that forks of the schema are not allowed and therefore needs to be handled manually. 

Imagine a case like this
```
V0.0 --> V0.1
```
Now we want to develop a V0.2 and as part of that we release a beta version with the schema.
```
V0.0 --> V0.1 -> V0.2(beta)
```
After having run the beta for a while we are ready to ship V0.2 and so to not lose the projects that was made during the beta. We can ship V0.2 with the following schema:
```
V0.0 --> V0.1 --> V0.2
                   /
   V0.2(beta) -----
```
Now any project made in previous versions and the beta can all be updated to V0.2

## To Do
 - [X] Generer node traversal
 - [ ] get/set changeset
 - [X] getters/setters
 - [X] Tag komentare med
 - [X] Tag doc komentare med
 - [x] Python version
 - [X] Add faker-rs
 - [ ] Add field getters/setters
 - [X] Add bencher
 - [X] Navngivning af edge, node, endpoint get/set
 - [X] import typer
 - [X] auto traversal result<iterator/option>
 - [X] Graf type
 - [X] Add comments
 - [x] Write readme
 - [x] Add examples
 - [x] Custom graph serializer/deserializer