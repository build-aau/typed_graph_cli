use super::SingleChange;
use crate::{ChangeSetResult, FieldPath};
use build_script_lang::schema::Schema;
use build_script_shared::error::{ParserResult, ParserSlimResult};
use build_script_shared::parsers::*;
use build_script_shared::{compose_test, InputMarkerRef, InputType};
use fake::Dummy;
use nom::bytes::complete::tag;
use nom::character::complete::*;
use nom::combinator::{eof, opt};
use nom::error::context;
use nom::multi::*;
use nom::sequence::*;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

pub type DefaultChangeset<'a> = ChangeSet<InputMarkerRef<'a>>;

#[derive(PartialEq, Eq, Debug, Hash, Default, Dummy)]
pub struct ChangeSet<I> {
    pub new_version: Ident<I>,
    pub old_version: Ident<I>,
    pub handler: Option<Ident<I>>,
    pub new_hash: u64,
    pub old_hash: u64,
    pub changes: Vec<SingleChange<I>>,
}

impl<I> ChangeSet<I> {
    pub fn new() -> ChangeSet<I>
    where
        I: Default,
    {
        ChangeSet {
            old_version: Ident::default(),
            new_version: Ident::default(),
            handler: None,
            old_hash: 0,
            new_hash: 0,
            changes: Vec::new(),
        }
    }

    pub fn extend(&mut self, other: ChangeSet<I>)
    where
        I: PartialEq + Debug,
    {
        self.changes.extend(other.changes)
    }

    pub fn push(&mut self, change: SingleChange<I>) {
        self.changes.push(change)
    }

    pub fn map<O, F>(self, f: F) -> ChangeSet<O>
    where
        F: Fn(I) -> O + Copy,
    {
        ChangeSet {
            old_hash: self.old_hash,
            new_hash: self.new_hash,
            handler: self.handler.map(|i| i.map(f)),
            old_version: self.old_version.map(f),
            new_version: self.new_version.map(f),
            changes: self.changes.into_iter().map(|c| c.map(f)).collect(),
        }
    }

    /// retrieve all changes that affect any part of the provided path
    pub fn get_changes(&self, path: FieldPath<I>) -> Vec<&SingleChange<I>>
    where
        I: PartialEq,
    {
        let mut changes = Vec::new();
        for change in &self.changes {
            match change {
                SingleChange::AddedType(f) => {
                    if f.type_name == path.root {
                        changes.push(change)
                    }
                }
                SingleChange::EditedType(f) => {
                    if f.type_name == path.root {
                        changes.push(change)
                    }
                }
                SingleChange::AddedEndpoint(f) => {
                    if f.type_name == path.root {
                        changes.push(change)
                    }
                }
                SingleChange::AddedField(f) => {
                    if f.field_path == path
                        || path.path.is_empty() && f.field_path.root == path.root
                    {
                        changes.push(change)
                    }
                }
                SingleChange::AddedVarient(f) => {
                    let is_changed = f.type_name == path.root
                        && path
                            .path
                            .get(0)
                            .map(|n| n == &f.varient_name)
                            .unwrap_or_else(|| true);

                    if is_changed {
                        changes.push(change)
                    }
                }
                SingleChange::EditedFieldType(f) => {
                    if f.field_path == path
                        || path.path.is_empty() && f.field_path.root == path.root
                    {
                        changes.push(change)
                    }
                }
                SingleChange::EditedOpaque(f) => {
                    if f.field_path == path
                        || path.path.is_empty() && f.field_path.root == path.root
                    {
                        changes.push(change)
                    }
                }
                SingleChange::EditedGenerics(f) => {
                    if f.type_name == path.root {
                        changes.push(change);
                    }
                }
                SingleChange::EditedSchema(_) => {
                    if path.root == "" {
                        changes.push(change)
                    }
                }
                SingleChange::EditedVariantsOrder(f) => {
                    if f.type_name == path.root {
                        changes.push(change);
                    }
                }
                SingleChange::EditedEndpoint(f) => {
                    if path.path.len() == 0 && f.type_name == path.root {
                        changes.push(change)
                    }
                }
                SingleChange::EditedVariant(f) => {
                    if path.path.len() == 0 && f.type_name == path.root {
                        changes.push(change)
                    }
                }
                SingleChange::RemovedField(f) => {
                    if f.field_path == path
                        || path.path.is_empty() && f.field_path.root == path.root
                    {
                        changes.push(change)
                    }
                }
                SingleChange::RemovedType(f) => {
                    if f.type_name == path.root {
                        changes.push(change)
                    }
                }
                SingleChange::RemovedEndpoint(f) => {
                    if f.type_name == path.root {
                        changes.push(change)
                    }
                }
                SingleChange::RemovedVarient(f) => {
                    let is_changed = f.type_name == path.root
                        && path
                            .path
                            .get(0)
                            .map(|n| n == &f.varient_name)
                            .unwrap_or_else(|| true);

                    if is_changed {
                        changes.push(change)
                    }
                }
            };
        }

        changes
    }

    pub fn apply(&self, schema: Schema<I>) -> ChangeSetResult<Schema<I>>
    where
        I: Hash + Clone + Default + PartialEq + Debug + Ord,
    {
        let old_hash = schema.get_hash();
        if old_hash != self.old_hash || schema.version != self.old_version {
            return Err(crate::ChangeSetError::IncompatibleSchemaVersion {
                expected: self.old_hash,
                recieved: old_hash,
                old_version: self.old_version.to_string(),
                new_version: self.new_version.to_string(),
            });
        }

        let mut updated_schema = schema.clone();
        updated_schema.version = self.new_version.clone();

        for change in &self.changes {
            change.apply(&mut updated_schema)?;
        }

        // Verify that the convertion was successful
        let new_hash = updated_schema.get_hash();

        if new_hash != self.new_hash {
            println!("Created schema:");
            println!("{}", updated_schema.serialize_to_string().unwrap());

            return Err(crate::ChangeSetError::UpdateFailed {
                expected: self.new_hash,
                recieved: new_hash,
                old_version: self.old_version.to_string(),
                new_version: self.new_version.to_string(),
            });
        }

        Ok(schema)
    }

    pub fn check_convertion_res(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        for change in &self.changes {
            change.check_convertion_res()?;
        }

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for ChangeSet<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, (((old_version, old_hash), (new_version, new_hash)), handler)) = context(
            "Parsing ChangeSet version",
            surrounded(
                '<',
                tuple((
                    separated_pair(
                        ws(pair(Ident::ident_full, ws(surrounded('(', hex_u64, ')')))),
                        pair(char('='), char('>')),
                        ws(pair(Ident::ident_full, ws(surrounded('(', hex_u64, ')')))),
                    ),
                    opt(preceded(
                        tuple((ws(char(',')), tag("handler"), ws(char('=')))),
                        Ident::ident,
                    )),
                )),
                '>',
            ),
        )(s)?;

        let (s, changes) = context(
            "Parsing ChangeSet changes",
            terminated(
                ws(many0(ws(SingleChange::parse))),
                context("Expected change", eof),
            ),
        )(s)?;

        let changeset = ChangeSet {
            old_version,
            new_version,
            handler,
            new_hash,
            old_hash,
            changes,
        };

        changeset.check_convertion_res()?;

        Ok((s, changeset))
    }
}

impl<I> ParserSerialize for ChangeSet<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        let changset_ctx = ctx.set_indents(0);
        write!(f, "{indents}< ")?;
        self.old_version.compose(f, changset_ctx)?;
        write!(f, "({:#16x}) => ", self.old_hash)?;
        self.new_version.compose(f, changset_ctx)?;
        write!(f, "({:#16x})", self.new_hash)?;
        if let Some(handler) = &self.handler {
            write!(f, ", handler = ")?;
            handler.compose(f, changset_ctx)?;
        }
        writeln!(f, " >")?;

        for change in &self.changes {
            change.compose(f, ctx)?;
            writeln!(f, "")?;
        }
        Ok(())
    }
}

impl<I: Hash> ChangeSet<I> {
    pub fn get_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
}

impl<I> Display for ChangeSet<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for change in &self.changes {
            writeln!(f, "{}", change)?;
        }

        Ok(())
    }
}

compose_test! {changeset_compose, ChangeSet<I>}

#[test]
fn schema_changeset_test() {
    use crate::ChangeSetBuilder;
    use build_script_shared::tests::display_diff;
    use build_script_shared::CodePreview;
    use fake::{Fake, Faker};

    for _ in 0..build_script_shared::tests::TEST_ITERATION_COUNT {
        let old_schema: Schema<String> = Faker.fake();
        let new_schema: Schema<String> = Faker.fake();

        let changes = old_schema.build_changeset(&new_schema).unwrap();

        println!("Found changes:");
        let change_showcase = changes.serialize_to_string().unwrap();
        println!("{}", CodePreview::showcase(change_showcase.clone()));

        let mut updated_schema = old_schema.clone();
        updated_schema.version = changes.new_version.clone();

        for change in &changes.changes {
            let res = change.apply(&mut updated_schema);
            if res.is_err() {
                println!();
                println!("Old schema:");
                println!(
                    "{}",
                    CodePreview::showcase(old_schema.serialize_to_string().unwrap())
                );
                println!();

                println!("Failed during execution of:");

                let of_interest = change.serialize_to_string().unwrap();
                let caret_len = of_interest.len();
                let caret_offset = change_showcase.find(&of_interest).unwrap_or_default();

                let preview = CodePreview::new(&change_showcase, caret_offset, caret_len, 4, 4, true);
                println!("{preview}");
                println!();
                res.unwrap();
            }
        }

        let mut hasher = DefaultHasher::new();
        new_schema.hash(&mut hasher);
        let new_hash = hasher.finish();

        let mut hasher = DefaultHasher::new();
        updated_schema.hash(&mut hasher);
        let updated_hash = hasher.finish();

        if &updated_schema != &new_schema {
            let mut dbg_updated_schema = updated_schema.clone();
            let mut dbg_new_schema = new_schema.clone();

            // Each shcema file can have their own set of comments
            // So we remove all non doc comments
            dbg_updated_schema.strip_comments();
            dbg_new_schema.strip_comments();

            let content_iter = dbg_updated_schema.iter().zip(dbg_new_schema.iter());
            println!("Running diff on content:");
            for (updated_stm, new_stm) in content_iter {
                if updated_stm != new_stm {
                    display_diff(new_stm, updated_stm)
                }
            }

            println!();
            println!("Running diff on schema:");
            display_diff(&dbg_new_schema, &dbg_updated_schema);
        }

        assert_eq!(updated_schema, new_schema);

        if updated_hash != new_hash {
            let content_iter = updated_schema.iter().zip(new_schema.iter());
            println!("Running diff on content:");
            for (updated_stm, new_stm) in content_iter {
                let mut hasher = DefaultHasher::new();
                new_stm.hash(&mut hasher);
                let new_hash = hasher.finish();

                let mut hasher = DefaultHasher::new();
                updated_stm.hash(&mut hasher);
                let updated_hash = hasher.finish();

                if updated_hash != new_hash {
                    println!(
                        "Hash mismatch {} -> {} for {} {}",
                        new_hash,
                        updated_hash,
                        updated_stm.get_schema_type(),
                        updated_stm.get_type()
                    );
                }
            }
        }

        assert_eq!(updated_hash, new_hash);
    }
}
