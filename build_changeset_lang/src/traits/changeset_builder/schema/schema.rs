use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use build_script_lang::schema::*;
use nom::Err;
use std::hash::Hash;

use crate::schema::*;
use crate::traits::ChangeSetBuilder;
use crate::ChangeSetResult;
use crate::{ChangeSetError, RemovedType};

impl<I> ChangeSetBuilder<I> for Schema<I>
where
    I: Clone + Hash + Default + PartialEq + Debug + Ord,
{
    fn build_changeset_with_path(
        &self,
        new_version: &Self,
        path: Option<FieldPath<I>>,
    ) -> ChangeSetResult<ChangeSet<I>> {
        if let Some(path) = path {
            return Err(ChangeSetError::InvalidFieldPath {
                path: path.to_string(),
                target: "Schema".to_string(),
            });
        }

        let mut old_types = HashMap::new();
        for stm in self.iter() {
            let type_name = stm.get_type();
            old_types.insert(type_name, stm);
        }

        let mut new_types = HashMap::new();
        for stm in new_version.iter() {
            let type_name = stm.get_type();
            new_types.insert(type_name, stm);
        }

        let old_keys: HashSet<_> = old_types.keys().collect();
        let new_keys: HashSet<_> = new_types.keys().collect();
        let edited_types_vec: Vec<_> = old_keys.intersection(&new_keys).collect();
        let removed_types_vec: Vec<_> = old_keys
            .difference(&new_keys)
            .map(|old_type_key| {
                let old_type = old_types.get(*old_type_key).unwrap();
                crate::schema::SingleChange::RemovedType(RemovedType {
                    type_type: old_type.get_schema_type(),
                    type_name: old_type.get_type().clone(),
                })
            })
            .collect();
        let added_types_vec: Vec<_> = new_keys.difference(&old_keys).collect();

        let mut added_types = ChangeSet::new();
        let mut removed_types = ChangeSet::new();
        let mut edited_types = ChangeSet::new();

        removed_types.changes = removed_types_vec;

        for new_type_key in added_types_vec {
            let new_type = new_types.get(*new_type_key).unwrap();
            let added = crate::schema::SingleChange::AddedType(AddedType {
                comments: new_type.get_comments().clone(),
                attributes: new_type.get_attributes().cloned().unwrap_or_default(),
                type_type: AddedTypeData::from_stm(new_type),
                type_name: new_type.get_type().clone(),
            });

            added_types.changes.push(added);

            let added_fields = match new_type {
                SchemaStm::Node(n) => {
                    let mut base = NodeExp::default();
                    base.name = n.name.clone();
                    base.build_changeset_with_path(n, None).unwrap()
                }
                SchemaStm::Struct(n) => {
                    let mut base = StructExp::default();
                    base.name = n.name.clone();
                    base.build_changeset_with_path(n, None).unwrap()
                }
                SchemaStm::Edge(n) => {
                    let mut base = EdgeExp::default();
                    base.name = n.name.clone();
                    base.endpoints = n.endpoints.clone();
                    base.build_changeset_with_path(n, None).unwrap()
                }
                SchemaStm::Enum(n) => {
                    let mut base = EnumExp::default();
                    base.name = n.name.clone();
                    base.build_changeset_with_path(n, None).unwrap()
                }
                SchemaStm::Import(n) => {
                    let mut base = ImportExp::default();
                    base.name = n.name.clone();
                    base.build_changeset_with_path(n, None).unwrap()
                }
            };

            added_types.extend(added_fields);
        }

        for edited_type in edited_types_vec {
            let old_type = old_types.get(*edited_type).unwrap();
            let new_type = new_types.get(*edited_type).unwrap();

            match (old_type, new_type) {
                (SchemaStm::Node(type0), SchemaStm::Node(type1)) => {
                    edited_types.extend(type0.build_changeset_with_path(type1, None)?)
                }
                (SchemaStm::Edge(type0), SchemaStm::Edge(type1)) => {
                    edited_types.extend(type0.build_changeset_with_path(type1, None)?)
                }
                (SchemaStm::Enum(type0), SchemaStm::Enum(type1)) => {
                    edited_types.extend(type0.build_changeset_with_path(type1, None)?)
                }
                (SchemaStm::Struct(type0), SchemaStm::Struct(type1)) => {
                    edited_types.extend(type0.build_changeset_with_path(type1, None)?)
                }
                (SchemaStm::Import(type0), SchemaStm::Import(type1)) => {
                    edited_types.extend(type0.build_changeset_with_path(type1, None)?)
                }
                // For all other types we default to just recreating the element
                (type0, type1) => {
                    removed_types.push(crate::schema::SingleChange::RemovedType(RemovedType {
                        type_type: type0.get_schema_type(),
                        type_name: type0.get_type().clone(),
                    }));
                    added_types.push(crate::schema::SingleChange::AddedType(AddedType {
                        comments: type1.get_comments().get_doc_comments(),
                        attributes: type1.get_attributes().cloned().unwrap_or_default(),
                        type_type: AddedTypeData::from_stm(type1),
                        type_name: type1.get_type().clone(),
                    }));
                }
            }
        }

        let mut changes = added_types;
        changes.extend(edited_types);
        changes.extend(removed_types);

        changes.old_hash = self.get_hash();
        changes.new_hash = new_version.get_hash();
        changes.old_version = self.version.clone();
        changes.new_version = new_version.version.clone();

        for change in &changes.changes {
            let mut field_path = None;
            if let SingleChange::EditedFieldType(edit) = change {
                field_path = Some(&edit.field_path);
            }
            let field_path_str =
                field_path.map_or_else(|| format!("<Missing path>"), |path| path.to_string());

            let res = change.check_convertion_res();
            if let Err(nom_err) = res {
                return match nom_err {
                    Err::Failure(parser_err) | Err::Error(parser_err) => {
                        if let Some((_, e)) = parser_err.errors.get(0) {
                            Err(ChangeSetError::InvalidTypeMigration {
                                old_version: changes.old_version.to_string(),
                                new_version: changes.new_version.to_string(),
                                reason: format!("{e}"),
                                path: field_path_str,
                            })
                        } else {
                            Err(ChangeSetError::InvalidTypeMigration {
                                old_version: changes.old_version.to_string(),
                                new_version: changes.new_version.to_string(),
                                reason: format!("Recieved no error information"),
                                path: field_path_str,
                            })
                        }
                    }
                    Err::Incomplete(needed) => Err(ChangeSetError::InvalidTypeMigration {
                        old_version: changes.old_version.to_string(),
                        new_version: changes.new_version.to_string(),
                        reason: format!("Missing more input {:?}", needed),
                        path: field_path_str,
                    }),
                };
            }
        }

        Ok(changes)
    }
}
