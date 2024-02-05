use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fmt::Debug;

use build_script_lang::schema::EdgeExp;
use build_script_lang::schema::Schema;
use build_script_lang::schema::SchemaStm;
use build_script_lang::schema::SchemaStmType;
use build_script_shared::CodePreview;
use build_script_shared::InputMarker;
use fake::Dummy;
use fake::Fake;
use build_script_shared::parsers::ParserSerialize;

use crate::schema::*;
use crate::traits::ChangeSetBuilder;
use crate::ChangeSetError;
use crate::ChangeSetResult;

impl<I> ChangeSetBuilder<I> for EdgeExp<I>
where
    I: Default + Clone + PartialEq + Debug,
{
    fn build_changeset_with_path(
        &self,
        new_version: &Self,
        path: Option<FieldPath<I>>,
    ) -> ChangeSetResult<ChangeSet<I>> {
        if let Some(path) = path {
            return Err(ChangeSetError::InvalidFieldPath {
                path: path.to_string(),
                target: self.name.to_string(),
            });
        }

        if self.name != new_version.name {
            return Err(ChangeSetError::InvalidTypeComparison {
                type0: self.name.to_string(),
                type1: new_version.name.to_string(),
            });
        }

        let old_keys: HashSet<_> = self.endpoints.keys().collect();
        let new_keys: HashSet<_> = new_version.endpoints.keys().collect();

        let mut changes = ChangeSet::new();

        let removed_endpoints = old_keys.difference(&new_keys);
        for key in removed_endpoints {
            let endpoint = self.endpoints.get(key).unwrap();
            changes.push(SingleChange::RemovedEndpoint(RemovedEndpoint {
                type_name: self.name.clone(),
                endpoint: (*endpoint).clone(),
            }));
        }

        let added_endpoints = new_keys.difference(&old_keys);
        for key in added_endpoints {
            let endpoint = new_version.endpoints.get(key).unwrap();
            changes.push(SingleChange::AddedEndpoint(AddedEndpoint {
                type_name: self.name.clone(),
                endpoint: (*endpoint).clone(),
            }));
        }

        let edited_endpoints = old_keys.intersection(&new_keys);
        for key in edited_endpoints {
            let old_endpoint = self.endpoints.get(&key).unwrap();
            let new_endpoint = new_version.endpoints.get(&key).unwrap();

            if old_endpoint.quantity != new_endpoint.quantity
                || old_endpoint.attributes != new_endpoint.attributes
            {
                changes.push(SingleChange::EditedEndpoint(EditedEndpoint {
                    type_name: self.name.clone(),
                    endpoint: new_endpoint.clone(),
                }));
            }
        }

        let new_path = Some(FieldPath::new(self.name.clone()));
        let field_changes = self
            .fields
            .build_changeset_with_path(&new_version.fields, new_path)?;
        changes.extend(field_changes);

        let new_doc_comments = new_version.comments.get_doc_comments();
        if &new_doc_comments != &self.comments.get_doc_comments()
            || new_version.attributes != self.attributes
        {
            changes.push(SingleChange::EditedType(EditedType {
                comments: new_doc_comments,
                attributes: new_version.attributes.clone(),
                type_type: SchemaStmType::Edge,
                type_name: self.name.clone(),
            }));
        }

        Ok(changes)
    }
}