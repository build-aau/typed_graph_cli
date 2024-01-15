use build_script_lang::schema::EnumExp;
use build_script_lang::schema::SchemaStmType;
use build_script_shared::parsers::Attributes;

use crate::ChangeSetError;
use crate::ChangeSetResult;
use crate::schema::*;
use crate::traits::ChangeSetBuilder;

impl<I> ChangeSetBuilder<I> for EnumExp<I>
where
    I: Clone + Default
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

        let mut changes: Vec<_> = new_version
            .varients
            .iter()
            .filter(|(varient, _)| !self.varients.contains_key(varient))
            .map(|(varient, comments)| SingleChange::AddedVarient(AddedVarient{
                comments: comments.get_doc_comments(),
                type_name: self.name.clone(),
                varient_name: varient.clone(),
            }))
            .collect();

        let removed_fields = self
            .varients
            .iter()
            .filter(|(varient, _)| !new_version.varients.contains_key(varient))
            .map(|(varient, _)| SingleChange::RemovedVarient(RemovedVarient{
                type_name: self.name.clone(),
                varient_name: varient.clone(),
            }));

        changes.extend(removed_fields);

        let new_doc_comments = new_version.comments.get_doc_comments();
        if &new_doc_comments != &self.comments.get_doc_comments() {
            changes.push(SingleChange::EditedType(EditedType { 
                comments: new_doc_comments, 
                attributes: Attributes::default(),
                type_type: SchemaStmType::Enum, 
                type_name: self.name.clone() 
            }));
        }

        Ok(ChangeSet {
            new_hash: Default::default(),
            old_hash: Default::default(),
            handler: None,
            new_version: Default::default(),
            old_version: Default::default(),
            changes
        })
    }
}
