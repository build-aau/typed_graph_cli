use build_changeset_lang::{ChangeSet, ChangeSetBuilder, DefaultChangeset};
use build_script_lang::schema::Schema;
use build_script_lang::DefaultSchema;
use build_script_shared::parsers::{Ident, Mark, ParserDeserialize, ParserSerialize};
use build_script_shared::InputMarker;
use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, read_dir, read_to_string, remove_file};
use std::path::{Path, PathBuf};

use crate::{GenError, GenResult};

#[derive(Default)]
pub struct Project {
    schemas: HashMap<String, Schema<InputMarker<String>>>,
    changesets: HashMap<u64, ChangeSet<InputMarker<String>>>,
    version_tree: HashMap<String, HashMap<String, (u64, Direction)>>,
    schema_folder: PathBuf,
    changeset_folder: PathBuf,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum Direction {
    Forward,
    Backwards,
}

impl Project {
    /// Create the project folder if it does not exist
    pub fn create_project<P: AsRef<Path>>(p: P) -> GenResult<Project> {
        let (schema_folder, changeset_folder) = Project::create_project_directory(p)?;
        Project::open(schema_folder, changeset_folder, true)
    }

    /// Open a project as normal
    pub fn open_project<P: AsRef<Path>>(p: P) -> GenResult<Project> {
        let (schema_folder, changeset_folder) = Project::open_project_directory(p)?;
        Project::open(schema_folder, changeset_folder, true)
    }

    /// Open a project without checking if the data makes sense
    pub fn open_project_raw<P: AsRef<Path>>(p: P) -> GenResult<Project> {
        let (schema_folder, changeset_folder) = Project::open_project_directory(p)?;
        Project::open(schema_folder, changeset_folder, false)
    }

    /// Internal method or opening a project  
    /// check_integrity can be set to run a check if the changesets point to valid schemas
    /// 
    /// A project is made of two parts:
    /// 1. a schema folder holding a number of schemas
    /// 2. a changeset folder holding a number of changesets between schemas 
    fn open(
        schema_folder: PathBuf,
        changeset_folder: PathBuf,
        check_integrity: bool,
    ) -> GenResult<Project> {
        let mut project = Project::default();

        project.schema_folder = schema_folder.clone();
        project.changeset_folder = changeset_folder.clone();

        project.load_schemas(schema_folder)?;
        project.load_changesets(changeset_folder)?;

        if check_integrity {
            project.verify_integrity()?;
        }

        Ok(project)
    }

    pub fn remove_changeset(&mut self, changeset_id: u64) -> GenResult<ChangeSet<InputMarker<String>>> {
        let changeset =
            self.changesets
                .remove(&changeset_id)
                .ok_or_else(|| GenError::UnknownChangeset {
                    name: changeset_id.clone(),
                })?;

        self.version_tree
            .entry(changeset.old_version.to_string())
            .or_default()
            .remove(changeset.new_version.as_str());
        self.version_tree
            .entry(changeset.new_version.to_string())
            .or_default()
            .remove(changeset.old_version.as_str());

        Ok(changeset)
    }

    /// Update the hashes of a changeset  
    /// This will also update the corresponding changeset file
    pub fn update_changeset(&mut self, changeset_id: &u64) -> GenResult<()> {
        let changeset = self.get_changeset(&changeset_id)?;
        let old_hash = self.get_schema(&changeset.old_version)?.get_hash();
        let new_hash = self.get_schema(&changeset.new_version)?.get_hash();

        let need_update = changeset.old_hash != old_hash || changeset.new_hash != new_hash;

        if !need_update {
            return Ok(());
        }

        let changset = self.remove_changeset(*changeset_id)?;
        let new_id = self.create_changeset(&changset.old_version, &changset.new_version)?;
        self.save_changeset(&new_id)?;

        Ok(())
    }

    pub fn get_schema_folder(&self) -> &Path {
        &self.schema_folder
    }

    pub fn get_changeset_folder(&self) -> &Path {
        &self.changeset_folder
    }

    pub fn has_schema(&self, id: &str) -> bool {
        self.schemas.contains_key(id)
    }

    pub fn get_schema(&self, id: &String) -> GenResult<&Schema<InputMarker<String>>> {
        self.get_schema_safe(id)
            .ok_or_else(|| GenError::UnknownSchema { name: id.clone() })
    }

    pub fn get_changeset(&self, id: &u64) -> GenResult<&ChangeSet<InputMarker<String>>> {
        self.get_changeset_safe(id)
            .ok_or_else(|| GenError::UnknownChangeset { name: id.clone() })
    }

    pub fn get_schema_safe(&self, id: &String) -> Option<&Schema<InputMarker<String>>> {
        self.schemas.get(id)
    }

    pub fn get_changeset_safe(&self, id: &u64) -> Option<&ChangeSet<InputMarker<String>>> {
        self.changesets.get(id)
    }

    pub fn iter_schema(&self) -> impl Iterator<Item = &String> {
        self.schemas.keys()
    }

    pub fn iter_changesets(&self) -> impl Iterator<Item = &u64> {
        self.changesets.keys()
    }

    /// Get an iterator over the version tree  
    /// The version tree contians both forward and backwards convertions of all chnagesets  
    /// Each entry is on the form (old_version, new_version, changset_id)
    /// 
    /// if dir is provided only changesets in a specific direction is included  
    pub fn iter_version(
        &self,
        dir: Option<Direction>,
    ) -> impl Iterator<Item = (&String, &String, &u64)> {
        self.version_tree
            .iter()
            .map(move |(old, tree)| {
                tree.iter()
                    .filter(move |(_, (_, tree_dir))| {
                        dir.map_or_else(|| false, |dir| dir == *tree_dir)
                    })
                    .map(move |(new, (id, _))| (old, new, id))
            })
            .flatten()
    }

    /// Create a copy of a schema  
    /// If the schema ends on a number, then it is incremented otherwise "_copy" i appended
    pub fn copy_schema(&mut self, id: &String, increment_name: bool) -> GenResult<String> {
        let old_schema = self.get_schema(id)?;
        let mut new_schema = old_schema.clone();

        let name = new_schema.version.to_string();
        let mut new_name = if increment_name {
            Project::increment_name(&name, || "_copy".chars())
        } else {
            name
        };

        while self.has_schema(&new_name) {
            new_name += "_copy";
        }

        new_schema.version = Ident::new(new_name, Mark::default());
        let new_version = new_schema.version.to_string();
        self.add_schema(new_schema.clone())?;
        self.save_schema(&new_version)?;

        Ok(new_version)
    }

    /// Attempts to increment a number at the end of a string
    fn increment_name<F, FR>(name: &String, default: F) -> String
    where
        F: Fn() -> FR,
        FR: Iterator<Item = char>,
    {
        let mut chars: Vec<char> = name
            .chars()
            .collect();
        let mut num = Vec::new();

        // Finds number
        while let Some(n) = chars.pop() {
            if n.is_numeric() {
                // This retrieves the number in reverse order
                num.push(n);
            } else {
                // Since we use pop we have to add back the last letter
                chars.push(n);
                break;
            }
        }

        if !num.is_empty() {

            let mut overflow = true;

            // you could image we had "Hello789"
            // num := ['9', '8', '7']
            // The order is reversed because we started from the end of the string
            // So we just increment each of them and check for overflow
            for c in &mut num {
                if !overflow {
                    break;
                }

                if *c == '9' {
                    // first iteration num := ['0', '8', '7']
                    overflow = true;
                    *c = '0';
                } else {
                    // second iteration num := ['0', '9', '7']
                    overflow = false;
                    *c = (*c as u8 + 1) as char;
                }
            }

            // Resolve overflow if any
            if overflow {
                num.push('1');
            }

            // The resulting is "Hello790" after reinsertion
            num.reverse();
            chars.extend(num);

        } else {
            // If no number was found use a default instead
            chars.extend(default());
        }

        chars.into_iter().collect()
    }

    /// Create a changeset between two schemas  
    /// This will also create the changeset file
    pub fn create_changeset(&mut self, old: &String, new: &String) -> GenResult<u64> {
        let old_schema = self.get_schema(&old)?;
        let new_schema = self.get_schema(&new)?;

        let changeset = self.version_tree.get(old).and_then(|inner| inner.get(new));

        if let Some((id, _)) = changeset {
            let changeset = self.get_changeset(id)?;
            return Err(GenError::DuplicateKeys {
                kind: "changeset".to_string(),
                old: changeset.old_version.to_string(),
                new: changeset.new_version.to_string(),
                old_hash: changeset.old_hash,
                new_hash: changeset.new_hash,
            });
        }

        let changeset = old_schema.build_changeset(new_schema)?;

        println!("Found changes from {old} to {new}:");
        println!("{}", changeset);

        let new = self.add_changeset(changeset)?;

        self.save_changeset(&new)?;

        Ok(new)
    }

    /// Save a schema to a file in the project folder
    pub fn save_schema(&self, schema: &String) -> GenResult<PathBuf> {
        let schema = self.get_schema(schema)?;
        let p = self.schema_folder.join(format!("{}.bs", schema.version));
        schema.serialize_to_file(&p)?;
        Ok(p)
    }

    /// Save a changeset to a file in the project folder
    pub fn save_changeset(&self, changset: &u64) -> GenResult<PathBuf> {
        let changeset = self.get_changeset(changset)?;
        self.test_changeset(changeset)?;
        let p = self.changeset_folder.join(format!(
            "{} {}.bs.diff",
            changeset.old_version, changeset.new_version
        ));
        changeset.serialize_to_file(&p)?;
        Ok(p)
    }

    /// Add a new schema to the project  
    /// THIS WILL NOT SAVE IT AS A FILE!!!!
    pub fn add_schema(&mut self, schema: Schema<InputMarker<String>>) -> GenResult<String> {
        let new_version = schema.version.to_string();
        let new_hash = schema.get_hash();
        let duplicate_key = self.schemas.insert(new_version.clone(), schema);

        if let Some(old_schema) = duplicate_key {
            return Err(GenError::DuplicateKeys {
                kind: "schema".to_string(),
                old: old_schema.version.to_string(),
                new: new_version,
                old_hash: old_schema.get_hash(),
                new_hash: new_hash,
            });
        }

        Ok(new_version)
    }

    /// Rename a schema and all changesets for it
    /// This will also update the files as needed
    pub fn rename_schema(&mut self, old_schema: &String, new_name: String) -> GenResult<()> {
        if !self.has_schema(old_schema) {
            return Err(GenError::UnknownSchema {
                name: old_schema.clone(),
            });
        }

        // Check for name collisions
        let schema_ref = self.get_schema(old_schema)?;
        if let Ok(new_schema) = self.get_schema(&new_name) {
            return Err(GenError::DuplicateKeys {
                kind: "schema".to_string(),
                old: schema_ref.version.to_string(),
                new: new_schema.version.to_string(),
                old_hash: schema_ref.get_hash(),
                new_hash: new_schema.get_hash(),
            });
        }

        let mut schema = self.schemas.remove(old_schema).unwrap();
        schema.version = Ident::new_alone(new_name.clone());
        let new_schema_hash = schema.get_hash();
        
        // Update the schema
        self.schemas.insert(new_name.clone(), schema);

        // Find the affected changesets
        let mut changesets_to_update = Vec::new();
        if let Some(changesets) = self.version_tree.get(old_schema) {
            for changeset in changesets.values() {
                changesets_to_update.push(changeset.clone());
            }
        }

        // Update the changeset
        let mut updated_changesets = Vec::new();
        for (old_changeset_id, dir) in &changesets_to_update {
            let mut changeset = self.changesets.remove(old_changeset_id).unwrap();

            let old_source = changeset.old_version.to_string();
            let old_target = changeset.new_version.to_string();

            let new_changeset_id = changeset.get_hash();

            // Figure out what has changed
            match dir {
                Direction::Forward => {
                    changeset.old_version = Ident::new_alone(new_name.clone());
                    changeset.old_hash = new_schema_hash;
                }
                Direction::Backwards => {
                    changeset.new_version = Ident::new_alone(new_name.clone());
                    changeset.new_hash = new_schema_hash;
                }
            }

            // Create an easy way to acces the updated values
            // We can then begin to update the project with the changed changesets
            let new_source = changeset.old_version.to_string();
            let new_target = changeset.new_version.to_string();

            // Register the updated changeset
            self.changesets.insert(new_changeset_id, changeset);
            let p = self
                .changeset_folder
                .join(format!("{} {}.bs.diff", old_source, old_target));
            updated_changesets.push((new_changeset_id, p));

            // Remove the old changeset in the version tree
            let forward = self.version_tree.entry(old_source.clone()).or_default();
            forward.remove(&old_target);

            if forward.is_empty() {
                self.version_tree.remove(&old_source);
            }

            let backwards = self.version_tree.entry(old_target.clone()).or_default();
            backwards.remove(&old_source);

            if backwards.is_empty() {
                self.version_tree.remove(&old_target);
            }

            // Add the new changest in the version tree
            self.version_tree
                .entry(new_source.clone())
                .or_default()
                .insert(new_target.clone(), (new_changeset_id, Direction::Forward));
            self.version_tree
                .entry(new_target.clone())
                .or_default()
                .insert(new_source.clone(), (new_changeset_id, Direction::Backwards));
        }

        // Update changeset files
        for (changeset_id, p) in updated_changesets {
            remove_file(p)?;
            self.save_changeset(&changeset_id)?;
        }

        // Update schema file
        let p = self.schema_folder.join(format!("{}.bs", old_schema));
        remove_file(p)?;
        self.save_schema(&new_name)?;

        Ok(())
    }

    
    fn add_changeset(&mut self, changeset: ChangeSet<InputMarker<String>>) -> GenResult<u64> {
        let id = changeset.get_hash();

        self.version_tree
            .entry(changeset.old_version.to_string())
            .or_default()
            .insert(changeset.new_version.to_string(), (id, Direction::Forward));
        self.version_tree
            .entry(changeset.new_version.to_string())
            .or_default()
            .insert(
                changeset.old_version.to_string(),
                (id, Direction::Backwards),
            );

        let duplicate_key: Option<ChangeSet<InputMarker<String>>> = self.changesets.insert(id, changeset);
        if let Some(changeset) = duplicate_key {
            return Err(GenError::DuplicateKeys {
                kind: "changset".to_string(),
                old: changeset.old_version.to_string(),
                new: changeset.new_version.to_string(),
                old_hash: changeset.old_hash,
                new_hash: changeset.new_hash,
            });
        }

        Ok(id)
    }

    /// Find heads in the version tree
    pub fn find_heads(&self) -> Vec<String> {
        let mut schema_keys: HashSet<String> = self.schemas.keys().cloned().collect();
        for (old, _, _) in self.iter_version(Some(Direction::Forward)) {
            schema_keys.remove(old);
        }
        schema_keys.into_iter().collect()
    }

    /// Find roots in the version tree
    pub fn find_roots(&self) -> Vec<String> {
        let mut schema_keys: HashSet<String> = self.schemas.keys().cloned().collect();
        for (old, _, _) in self.iter_version(Some(Direction::Backwards)) {
            schema_keys.remove(old);
        }
        schema_keys.into_iter().collect()
    }

    /// Check if the project folder exists
    fn open_project_directory<P: AsRef<Path>>(p: P) -> GenResult<(PathBuf, PathBuf)> {
        let root = p.as_ref();
        if !root.exists() {
            return Err(GenError::InvalidProjectPath(root.to_path_buf()));
        }

        let schema_folder = root.join("schemas");
        let changeset_folder = root.join("changesets");

        if !schema_folder.exists() || !schema_folder.is_dir() {
            return Err(GenError::MissingFolder {
                folder: schema_folder
                    .to_str()
                    .ok_or_else(|| GenError::MalformedPath)?
                    .to_string(),
            });
        }

        if !changeset_folder.exists() || !changeset_folder.is_dir() {
            return Err(GenError::MissingFolder {
                folder: changeset_folder
                    .to_str()
                    .ok_or_else(|| GenError::MalformedPath)?
                    .to_string(),
            });
        }

        Ok((schema_folder, changeset_folder))
    }

    /// Create the project folder and the path to the folder if it does not exist
    fn create_project_directory<P: AsRef<Path>>(p: P) -> GenResult<(PathBuf, PathBuf)> {
        let root = p.as_ref();
        if !root.exists() {
            return Err(GenError::InvalidProjectPath(root.to_path_buf()));
        }

        let schema_folder = root.join("schemas");
        let changeset_folder = root.join("changesets");

        if !schema_folder.exists() {
            create_dir_all(&schema_folder)?;
        }

        if !changeset_folder.exists() {
            create_dir_all(&changeset_folder)?;
        }

        Ok((schema_folder, changeset_folder))
    }

    /// Load all schemas files from a folder
    fn load_schemas<P: AsRef<Path>>(&mut self, schema_folder: P) -> GenResult<()> {
        let schema_iter = read_dir(schema_folder)?;
        for schema_file in schema_iter {
            let schema_path = schema_file?;
            if let Ok(s) = schema_path.file_name().into_string() {
                if s.ends_with(".bs") {
                    let content = read_to_string(schema_path.path())?;
                    let input = InputMarker::new_from_file(
                        content.as_str(),
                        schema_path
                            .path()
                            .to_str()
                            .ok_or_else(|| GenError::MalformedPath)?
                            .to_string(),
                    );
                    let schema = DefaultSchema::deserialize(input)?;
                    let owned_schema = schema.map(|i| i.map(|data| data.to_string()));
                    self.add_schema(owned_schema)?;
                }
            }
        }

        Ok(())
    }

    /// Load all changeset files from a folder
    fn load_changesets<P: AsRef<Path>>(&mut self, changeset_folder: P) -> GenResult<()> {
        let changeset_iter = read_dir(changeset_folder)?;
        for changeset_file in changeset_iter {
            let changeset_path = changeset_file?;
            if let Ok(s) = changeset_path.file_name().into_string() {
                if s.ends_with(".bs.diff") {
                    let content = read_to_string(changeset_path.path())?;
                    let input = InputMarker::new_from_file(
                        content.as_str(),
                        changeset_path
                            .path()
                            .to_str()
                            .ok_or_else(|| GenError::MalformedPath)?
                            .to_string(),
                    );
                    let changeset = DefaultChangeset::deserialize(input)?;
                    let owned_changeset = changeset.map(|i| i.map(|data| data.to_string()));
                    self.add_changeset(owned_changeset)?;
                }
            }
        }
        Ok(())
    }

    /// If we find a changeset without a schema, we can rebuild the schema using the old schema
    pub fn build_missing_schemas(&mut self) -> GenResult<()> {
        // First figure out which we have changesets for but no actual implementation
        let mut missing_schema = Vec::new();
        for (_, new, _) in self.iter_version(Some(Direction::Forward)) {
            if !self.schemas.contains_key(new) {
                missing_schema.push(new.to_owned());
            }
        }

        // Next we make a list of all the changesets that should be used to create the missing schemas
        let mut process_queue = Vec::new();
        let mut processed: HashSet<String> = self.schemas.keys().cloned().collect();
        while let Some(missing) = missing_schema.pop() {
            if processed.contains(&missing) {
                continue;
            }
            // Find a parent schema to the current one
            let all_paths = &self.version_tree[&missing];
            let changeset_id = all_paths
                .iter()
                .filter(|(_, (_, dir))| dir == &Direction::Backwards)
                .map(|(_, (id, _))| id)
                .next()
                .ok_or_else(|| GenError::UnreachableSchema {
                    target: missing.clone(),
                })?;

            // add the changeset to the  workload
            let changeset = &self.changesets[changeset_id];
            assert_eq!(changeset.new_version.to_string(), missing);
            processed.insert(missing);
            process_queue.push(*changeset_id);

            // Make sure the parent is created before the current one
            missing_schema.push(changeset.old_version.to_string());
        }

        // Finally we apply the changesets to generate the missing schemas
        while let Some(changeset_id) = process_queue.pop() {
            let changeset = &self.changesets[&changeset_id];
            if self
                .schemas
                .contains_key(&changeset.new_version.to_string())
            {
                continue;
            }

            let old_schema = &self.schemas[&changeset.old_version.to_string()];
            let new_schema = changeset.apply(old_schema.clone())?;
            let new = self.add_schema(new_schema)?;
            self.save_schema(&new)?;
        }

        Ok(())
    }

    /// Check if the loaded data makes sense
    fn verify_integrity(&self) -> GenResult<()> {
        self.check_version_tree()?;
        self.check_changset_hashes()?;
        Ok(())
    }

    /// Check if changes has been made to the schemas without updating the changesets
    fn check_changset_hashes(&self) -> GenResult<()> {
        for changeset in self.changesets.values() {
            self.test_changeset(changeset)?;
        }

        Ok(())
    }

    /// Check the hashes and correctness of a changeset
    fn test_changeset(&self, changeset: &ChangeSet<InputMarker<String>>) -> GenResult<()> {
        let old = self.get_schema(&changeset.old_version)?;
        let new = self.get_schema(&changeset.new_version)?;

        let old_hash = old.get_hash();
        let new_hash = new.get_hash();

        if old_hash != changeset.old_hash {
            return Err(GenError::DivergentChangeset {
                old_version: changeset.old_version.to_string(),
                new_version: changeset.new_version.to_string(),
                schema: changeset.old_version.to_string(),
                expected: changeset.old_hash,
                recieved: old_hash,
            });
        }

        if new_hash != changeset.new_hash {
            return Err(GenError::DivergentChangeset {
                old_version: changeset.old_version.to_string(),
                new_version: changeset.new_version.to_string(),
                schema: changeset.new_version.to_string(),
                expected: changeset.new_hash,
                recieved: new_hash,
            });
        }

        changeset.apply(old.clone())?;

        Ok(())
    }

    /// Check if version tree contain refences to non existing schemas or changesets
    fn check_version_tree(&self) -> GenResult<()> {
        for (old, new_tree) in &self.version_tree {
            for (new, (change_id, _)) in new_tree {
                if !self.changesets.contains_key(change_id) {
                    return Err(GenError::MalformedVersionTree {
                        kind: "changset".to_string(),
                        missing_key: format!("{:#16x}", change_id),
                    });
                }

                if !self.schemas.contains_key(new) {
                    return Err(GenError::MalformedVersionTree {
                        kind: "new schema".to_string(),
                        missing_key: new.clone(),
                    });
                }

                if !self.schemas.contains_key(old) {
                    return Err(GenError::MalformedVersionTree {
                        kind: "old schema".to_string(),
                        missing_key: old.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}