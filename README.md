# typed_graph_cli
Introduces BUILDscript a declarative language for creating staticly typed schemas for [TypedGraphs][typed_graph-git] and maintaining a history schemas.

[typed_graph-git]: https://github.com/build-aau/typed_graph "typed_graph latest"
## Getting started
migration_handler is mainly controled using a command line interface. However all functionlities in the cli is also available throught the cli module in the typed_graph_cli (currently not available in crates.io).

To create a new project run
```
migration_handler new
```
This creates two folders schemas and changesets.  
the schemas folder contains all the BUILDscript files with the extension *.bs  
the changesets folder contains all the Changesets with exstension *.bs.diff

### Editing schemas
all schemas are found in the project/schemas folder
The empty projects contian an enpty schema in project/schemas/v0.0.bs
```
<V0.0>
```
We then edit it to look like
```
<V0.0>
node Student{
    name: String,
    age: usize
};
node Classes{
    name: String
};
// A student can attend n classes
edge Attendance(Student => Classes) {
    grades: Option<Grades>
};

enum Grades {
    A,
    B,
    C,
    D,
    E,
    F
};
```
In this schema we describe a school with students and classes.  
Each student attend a number and classes and can be assigned a grade for those classes between A-F.

Next we will use this schema in either rust or python
### Generating Code
#### Rust
We first need a place to store the exported schema.
For this purpose we new crate.

```
cargo new --bin my_first_typed_graph
```

This will create the following files:
```
|-- project
|   |-- schemas
|   |-- changesets
|-- my_first_typed_graph
|   |-- src
|   |   |-- lib.rs
|   |-- Cargo.toml
```

We then add typed_graph to the Cargo.toml file as so
```toml
[package]
name = "my_first_typed_graph"
version = "0.1.0"
edition = "2021"

[dependencies]
typed_graph = "^0.1.0"
```

We can the generate the schema and store it in a folder inside the src directory
```
migration_handler export rust my_first_typed_graph/src/graph
```

The generated schema consist of a number of submodules with one for ever schema defined in the schemas folder
> NOTE: migration_handler will only ever create new files. It never delete old ones.  
so making changes to the schema files that does not exist anymore should be deleted manually

> NOTE: migration_handler will not ovewrite the content of imports files. This allows the schema to import external types
```
|-- project
|   |-- schemas
|   |-- changesets
|-- my_first_typed_graph
|   |-- src
|   |   |-- graph
|   |   |   |-- v0_0
|   |   |   |   |-- nodes              <-- Folder containing all node types
|   |   |   |   |   |-- student.rs     <-- Implentation of Student
|   |   |   |   |   |-- classes.rs     <-- Implementation of Classes
|   |   |   |   |   |-- mod.rs
|   |   |   |   |-- edges
|   |   |   |   |   |-- attendance.rs  <-- Implementation of Attencande
|   |   |   |   |   |-- mod.rs
|   |   |   |   |-- structs            <-- Folder containing all struct type
|   |   |   |   |-- types              <-- Folder containing all enum types
|   |   |   |   |   |-- grades.rs      <-- Implementation of Grades
|   |   |   |   |   |-- mod.rs
|   |   |   |   |-- schema.rs          <-- Implementation of V0_0 schema and V0_0Graph
|   |   |   |   |-- nodes.rs           <-- Container for all nodes
|   |   |   |   |-- edges.rs           <-- Container for all edges
|   |   |   |   |-- node_type.rs       <-- Container for all nodes without data
|   |   |   |   |-- edge_type.rs       <-- Container for all edges without data
|   |   |   |   |-- imports.rs         <-- Place to insert imports specific for V0.0
|   |   |   |   |-- mod.rs
|   |   |   |-- imports.rs             <-- Place to insert global imports
|   |   |   |-- mod.rs
|   |   |-- main.rs
|   |-- Cargo.toml
```

Finaly we can use the schema in main.rs

```rust
mod graph;

use crate::graph::v0_0::*;

type NodeKey = usize;
type EdgeKey = usize;

fn main() -> SchemaResult<(), NodeKey, EdgeKey, V0_1<NodeKey, EdgeKey>> {
    let mut g = V0_1Graph::default();

    let alice = Student::new(0, "Alice".to_string(), 21);
    let bob = Student::new(1, "Bob".to_string(), 21);

    let alice_id = g.add_node(alice)?;
    let bob_id = g.add_node(bob)?;

    let math_class = Classes::new(2, "Math".to_string());
    let english_class = Classes::new(3, "English".to_string());
    let biology_class = Classes::new(4, "Biology".to_string());

    let math_id = g.add_node(math_class)?;
    let english_id = g.add_node(english_class)?;
    let biology_id = g.add_node(biology_class)?;

    g.add_edge(alice_id, math_id, Attendance::new(0, None))?;
    g.add_edge(alice_id, english_id, Attendance::new(1, None))?;
    g.add_edge(alice_id, biology_id, Attendance::new(2, None))?;

    g.add_edge(bob_id, math_id, Attendance::new(3, None))?;
    g.add_edge(bob_id, english_id, Attendance::new(4, None))?;
    g.add_edge(bob_id, english_id, Attendance::new(5, Some(Grades::E)))?;
    Ok(())
}
```
#### Python
We first need a place to store the exported schema.  
For this purpose we create a new folder called my_first_typed_graph.

This should something like this:
```
|-- project
|   |-- schemas
|   |-- changesets
|-- my_first_typed_graph
|   |-- src
|   |   |-- main.py
```

Next we need to make sure typed_graph is installed
```
pyp install typed_graph
```

We can the generate the schema and store it inside the src directory
```
migration_handler export python my_first_typed_graph/src/graph
```

The generated schema consist of a number of submodules with one for ever schema defined in the schemas folder
> NOTE: migration_handler will only ever create new files. It never delete old ones.
> so making changes to the schema files that does not exist anymore should be deleted manually

> NOTE: migration_handler will not ovewrite the content of imports files. This allows the schema to import external types
```
|-- project
|   |-- schemas
|   |-- changesets
|-- my_first_typed_graph
|   |-- src
|   |   |-- graph
|   |   |   |-- v0_0
|   |   |   |   |-- nodes            <-- Folder containing all node types
|   |   |   |   |   |-- student.py   <-- Implentation of Student
|   |   |   |   |   |-- classes.py   <-- Implementation of Classes
|   |   |   |   |   |-- __init__py
|   |   |   |   |-- edges
|   |   |   |   |   |-- attendance.py <-- Implementation of Attendance
|   |   |   |   |   |-- __init__py
|   |   |   |   |-- structs           <-- Folder containing all struct type
|   |   |   |   |-- types             <-- Folder containing all enum types
|   |   |   |   |   |-- grades.py     <-- Implementation of Grades enum
|   |   |   |   |   |-- __init__py
|   |   |   |   |-- schema.py         <-- Implementation of V0_0 schema and V0_0Graph
|   |   |   |   |-- nodes.py          <-- Container for all nodes
|   |   |   |   |-- edges.py          <-- Container for all edges
|   |   |   |   |-- node_type.py      <-- Container for all nodes without data
|   |   |   |   |-- edge_type.py      <-- Container for all edges without data
|   |   |   |   |-- imports.py        <-- Place to insert imports specific for V0.0
|   |   |   |   |-- __init__py
|   |   |   |-- imports.py            <-- Place to insert global imports
|   |   |   |-- __init__py
|   |   |-- main.py
```

Next we need to specify which type we will use for ids.
This is done in my_first_typed_graph/src/graph/imports.py
```
NodeId = int
EdgeId = int
```

Finaly we can use the schema in main.py

```rust
from src.graph.v0_0 import *

g = Graph(V0_0())

alice = Student(id = 0, name = 'Alice', age = 21)
bob = Student(id = 1, name = 'Bob', age = 22)

alice_id = g.add_node(alice)
bob_id = g.add_node(bob)

math_class = Classes(id = 2, name = 'Math')
english_class = Classes(id = 3, name = 'English')
biology_class = Classes(id = 4, name = 'Biology')

math_id = g.add_node(math_class)
english_id = g.add_node(english_class)
biology_id = g.add_node(biology_class)

g.add_edge(alice_id, math_id, Attendance(id = 0, grades = None))
g.add_edge(alice_id, english_id, Attendance(id = 1, grades = None))
g.add_edge(alice_id, biology_id, Attendance(id = 2, grades = None))

g.add_edge(bob_id, math_id, Attendance(id = 3, grades = None))
g.add_edge(bob_id, english_id, Attendance(id = 4, grades = None))
g.add_edge(bob_id, english_id, Attendance(id = 5, grades = Grades.E))


```
### Working with changesets
a changeset between two schemas can be made using 

```
migration_handler migration link V0.0 V0.1
```
Where V0.0 and V0.1 is the name of the schema in the top of the schema file.  
Once a changeset is made the program will fail if any of the two schemas changes. 

> When updating the program changesets might become invalid.  
To handle this run `migration update` or `migration update -a`

Removing a changeset is done by deleting the corresponding file in the changesets folder.

The changesets must form a tree. Meaning cycles are not allowed.  
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

The migration can then be applied as
```rust
let g0_0 = V0_0Graph::default();
let g0_2 = g0_0.migrate_direct::<V0_2>();
```