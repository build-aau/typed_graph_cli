use std::fmt::Debug;

use build_script_lang::schema::NodeExp;
use build_script_lang::schema::SchemaStmType;

use crate::ChangeSetError;
use crate::ChangeSetResult;
use crate::schema::*;
use crate::traits::ChangeSetBuilder;

impl<I> ChangeSetBuilder<I> for NodeExp<I>
where
    I: Clone + Default + PartialEq + Debug
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
        
        let new_path = Some(FieldPath::new(self.name.clone()));
        let field_changes = self.fields.build_changeset_with_path(&new_version.fields, new_path)?;
        changes.extend(field_changes);

        let new_doc_comments = new_version.comments.get_doc_comments();
        if &new_doc_comments != &self.comments.get_doc_comments() {
            changes.push(SingleChange::EditedType(EditedType { 
                comments: new_doc_comments, 
                attributes: self.attributes.clone(),
                type_type: SchemaStmType::Node, 
                type_name: self.name.clone() 
            }));
        }
        

        Ok(changes)
    }
}
