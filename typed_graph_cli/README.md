# typed_graph_cli
Provides a command line interface for auto generating typed_graph interfaces for rust, python and possibly others in the future.

# Getting started
To create a new project run
```
> typed_graph_cli new
```


# Concepts
The cli works on a projects directory. Each project consist of a number of schemas and changesets each in their own respective folders. Each schema describes the allowed nodes and edges in a typed_graph with the changesets showing how the schema changes as it moves to other versions of the schema.

Once a changeset is made between two schemas the schemas cannot be changed anymore. The freezing of the schemas are done by calculating a hash of the data within the schema
Here is an example of this is action:

```
// This is our basline schema
<"V0.0">
node A {
    field1: String,
    field2: String
}
struct B {};
```

```
// This is a different schema, but the two can be considered equal since they both contains two fields with the same names and types
<"V0.0">
node A {
    // The fields are swapped, however the order of fields doesn't really impact their use
    field2: String
    field1: String,
}
struct B {};
```

```
// In this example node A is parsed after B. Which will change both their markers.
// However this does not affect the generated code and the schema can still be considered equivalent to the baseline
<"V0.0">
struct B {};
node A {
    field1: String,
    field2: String
}
```

When changes are detected to frozen schemas the cli will respond with an error until either the schema is reverted or the changesets are removed. The result is that a frozen version history is created for the schema leaving only the head nodes as mutable.

In the version tree each schema is allowed to have multiple parents, but only one child. This limitation is in place, to ensure that there only ever exist one path to migrate one schema to another version.


Once project the project has been populated with schemas it can be used to auto generate typed_graph interfaces for a number of target platforms.
All code generation is done by hand without the use of a templating engine. 
This is done in order to make it easier to trace where data comes.

Currently only Rust is supported but soon a read only version of python will be added.