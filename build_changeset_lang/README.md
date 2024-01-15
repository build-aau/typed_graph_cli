# build_changeset_lang
build_changeset_lang contains all functionality related to parsing and generating changesets based on a schema defined in build_script_lang

## Purpose
The changeset between two schemas is a list of instruction of what changes need to be made to the schema in order to go from one version to another.

Each changeset is not reversable and so only meant to update old schemas into new ones.
