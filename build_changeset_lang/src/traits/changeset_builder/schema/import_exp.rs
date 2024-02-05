use std::fmt::Debug;

use build_script_lang::schema::ImportExp;
use build_script_lang::schema::SchemaStmType;

use crate::schema::*;
use crate::traits::ChangeSetBuilder;
use crate::ChangeSetError;
use crate::ChangeSetResult;

impl<I> ChangeSetBuilder<I> for ImportExp<I>
where
    I: Clone + Default + PartialEq + Debug,
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

        let mut changes = ChangeSet::new();

        let new_doc_comments = new_version.comments.get_doc_comments();
        if self.comments.get_doc_comments() != new_doc_comments {
            changes.push(SingleChange::EditedType(EditedType {
                comments: new_doc_comments,
                attributes: Default::default(),
                type_type: SchemaStmType::Import,
                type_name: self.name.clone()
            }));
        }

        Ok(changes)
    }
}
