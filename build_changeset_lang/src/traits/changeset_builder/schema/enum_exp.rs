use build_script_lang::schema::EnumExp;
use build_script_lang::schema::EnumVarient;
use build_script_lang::schema::Fields;
use build_script_lang::schema::SchemaStmType;
use build_script_shared::parsers::Attributes;
use build_script_shared::parsers::Types;

use crate::schema::*;
use crate::traits::ChangeSetBuilder;
use crate::ChangeSetError;
use crate::ChangeSetResult;

impl<I> ChangeSetBuilder<I> for EnumExp<I>
where
    I: Clone + Default + PartialEq,
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

        // Check which varients are new
        let mut added_varients = Vec::new();
        let new_varient_iter = new_version.varients.iter().enumerate();
        for (order, varient) in new_varient_iter {
            if self.has_varient(varient.name()) {
                continue;
            }

            let attributes = match varient {
                EnumVarient::Struct { attributes, .. }
                | EnumVarient::Opaque { attributes, .. }
                | EnumVarient::Unit { attributes, .. } => attributes
            };

            added_varients.push(SingleChange::AddedVarient(AddedVarient {
                attributes: attributes.clone(),
                comments: varient.comments().get_doc_comments(),
                type_name: self.name.clone(),
                varient_name: varient.name().clone(),
                varient_type: AddedVarientType::from(varient),
                order: order as u64,
            }));

            if let EnumVarient::Struct { fields, name, .. } = varient {
                let new_path = FieldPath::new_path(self.name.clone(), vec![name.clone()]);

                let field_changes = Fields::default().build_changeset_with_path(fields, Some(new_path))?;
                
                for change in field_changes.changes {
                    added_varients.push(change);
                }
            };

        }

        // Check which varients are old
        let removed_varients = self
            .varients
            .iter()
            .filter(|varient| !new_version.has_varient(varient.name()))
            .map(|varient| {
                SingleChange::RemovedVarient(RemovedVarient {
                    type_name: self.name.clone(),
                    varient_name: varient.name().clone(),
                })
            });

        // And finally which varients has changed
        let mut edited_varients = Vec::new();
        let varient_iter = self.varients.iter().enumerate();
        for (order, varient) in varient_iter {
            let new_varient_order = new_version.get_varient(varient.name());
            if new_varient_order.is_none() {
                continue;
            }
            if let Some(new_vairent) = new_varient_order {
                let new_path = FieldPath::new_path(self.name.clone(), vec![varient.name().clone()]);

                match (varient, new_vairent) {
                    (
                        EnumVarient::Struct {
                            fields: fields1, ..
                        },
                        EnumVarient::Struct {
                            fields: fields2, ..
                        },
                    ) => {
                        let field_changes =
                            fields1.build_changeset_with_path(&fields2, Some(new_path))?;

                        for change in field_changes.changes {
                            edited_varients.push(change);
                        }

                        let new_doc_comments = new_version.comments.get_doc_comments();
                        if &new_doc_comments != &self.comments.get_doc_comments() {
                            edited_varients.push(SingleChange::EditedType(EditedType {
                                comments: new_doc_comments,
                                attributes: Attributes::default(),
                                type_type: SchemaStmType::Enum,
                                type_name: self.name.clone(),
                            }));
                        }
                    }
                    (EnumVarient::Opaque { .. }, EnumVarient::Unit { attributes, .. })
                    | (EnumVarient::Struct { .. }, EnumVarient::Unit { attributes, .. }) => {
                        edited_varients.push(SingleChange::RemovedVarient(RemovedVarient {
                            type_name: self.name.clone(),
                            varient_name: varient.name().clone(),
                        }));
                        edited_varients.push(SingleChange::AddedVarient(AddedVarient {
                            attributes: attributes.clone(),
                            comments: new_vairent.comments().get_doc_comments(),
                            type_name: self.name.clone(),
                            varient_name: varient.name().clone(),
                            varient_type: AddedVarientType::Unit,
                            order: order as u64,
                        }));
                    }
                    (EnumVarient::Opaque { .. }, EnumVarient::Struct { fields, attributes, .. })
                    | (EnumVarient::Unit { .. }, EnumVarient::Struct { fields, attributes, .. }) => {
                        edited_varients.push(SingleChange::RemovedVarient(RemovedVarient {
                            type_name: self.name.clone(),
                            varient_name: varient.name().clone(),
                        }));
                        edited_varients.push(SingleChange::AddedVarient(AddedVarient {
                            attributes: attributes.clone(),
                            comments: new_vairent.comments().get_doc_comments(),
                            type_name: self.name.clone(),
                            varient_name: varient.name().clone(),
                            varient_type: AddedVarientType::Struct,
                            order: order as u64,
                        }));

                        let field_changes =
                            Fields::default().build_changeset_with_path(fields, Some(new_path))?;

                        for change in field_changes.changes {
                            edited_varients.push(change);
                        }
                    }
                    (EnumVarient::Unit { .. }, EnumVarient::Unit { .. }) => {
                        let new_doc_comments = new_version.comments.get_doc_comments();
                        if &new_doc_comments != &self.comments.get_doc_comments() {
                            edited_varients.push(SingleChange::EditedType(EditedType {
                                comments: new_doc_comments,
                                attributes: Attributes::default(),
                                type_type: SchemaStmType::Enum,
                                type_name: self.name.clone(),
                            }));
                        }
                    }
                    (EnumVarient::Unit { .. }, EnumVarient::Opaque { ty, attributes, .. }) => {
                        edited_varients.push(SingleChange::RemovedVarient(RemovedVarient {
                            type_name: self.name.clone(),
                            varient_name: varient.name().clone(),
                        }));
                        edited_varients.push(SingleChange::AddedVarient(AddedVarient {
                            attributes: attributes.clone(),
                            comments: new_vairent.comments().get_doc_comments(),
                            type_name: self.name.clone(),
                            varient_name: varient.name().clone(),
                            varient_type: AddedVarientType::Opaque(ty.clone()),
                            order: order as u64,
                        }));
                    }
                    (EnumVarient::Struct { .. }, EnumVarient::Opaque { ty, attributes, .. }) => {
                        edited_varients.push(SingleChange::RemovedVarient(RemovedVarient {
                            type_name: self.name.clone(),
                            varient_name: varient.name().clone(),
                        }));
                        edited_varients.push(SingleChange::AddedVarient(AddedVarient {
                            attributes: attributes.clone(),
                            comments: new_vairent.comments().get_doc_comments(),
                            type_name: self.name.clone(),
                            varient_name: varient.name().clone(),
                            varient_type: AddedVarientType::Opaque(ty.clone()),
                            order: order as u64,
                        }));
                    }
                    (EnumVarient::Opaque { .. }, EnumVarient::Opaque { ty, attributes, .. }) => {
                        // TODO: Get this to modify the data not jsut replace it
                        edited_varients.push(SingleChange::RemovedVarient(RemovedVarient {
                            type_name: self.name.clone(),
                            varient_name: varient.name().clone(),
                        }));
                        edited_varients.push(SingleChange::AddedVarient(AddedVarient {
                            attributes: attributes.clone(),
                            comments: new_vairent.comments().get_doc_comments(),
                            type_name: self.name.clone(),
                            varient_name: varient.name().clone(),
                            varient_type: AddedVarientType::Opaque(ty.clone()),
                            order: order as u64,
                        }));
                    }
                }
            }
        }

        // Check which varients has moved
        let old_varient_order: Vec<_> = self.varients.iter().map(|v| v.name()).cloned().collect();

        let new_varient_order: Vec<_> = new_version
            .varients
            .iter()
            .map(|v| v.name())
            .cloned()
            .collect();

        let mut edited_varient_order = Vec::new();
        if old_varient_order != new_varient_order {
            edited_varient_order.push(SingleChange::EditedVariantsOrder(EditedVariantsOrder {
                type_name: self.name.clone(),
                old_order: old_varient_order,
                new_order: new_varient_order,
            }));
        }

        // Check if the generic has changed
        let mut edited_type = Vec::new();
        if self.generics != new_version.generics {
            edited_type.push(SingleChange::EditedGenerics(EditedGenerics {
                type_name: self.name.clone(),
                old_generics: self.generics.clone(),
                new_generics: new_version.generics.clone(),
            }));
        }

        for varient in &self.varients {
            if let Some(new_varient) = new_version.get_varient(varient.name()) {
                if varient.comments() != new_varient.comments() || varient.attributes() != new_varient.attributes() {
                    edited_type.push(SingleChange::EditedVariant(EditedVariant {
                        type_name: self.name.clone(),
                        varient_name: varient.name().clone(),
                        comments: new_varient.comments().get_doc_comments(),
                        attributes: new_varient.attributes().clone(),
                    }));
                }
            }
        }

        // Store the changes in a sorted order
        // this prevents a case where we edit a varient that does not exist
        // or try to sort a varient before it is created
        let mut changes: Vec<_> = Vec::new();
        changes.extend(added_varients);
        changes.extend(edited_varients);
        changes.extend(edited_varient_order);
        changes.extend(removed_varients);
        changes.extend(edited_type);

        Ok(ChangeSet {
            new_hash: Default::default(),
            old_hash: Default::default(),
            handler: None,
            new_version: Default::default(),
            old_version: Default::default(),
            changes,
        })
    }
}
