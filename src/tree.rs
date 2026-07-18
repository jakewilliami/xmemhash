//! Build a tree structure from a flat list of archive entries
//!
//! Archive handlers return a flat [`Vec<ArchiveEntry>`] where directories and files are siblings.  We need to compile this into a tree structure

use crate::archive::{ArchiveEntry, EntryData};
use std::path::{Component, PathBuf};

// Assemble tree from flat list by matching each entry's path components against
// existing directory nodes
pub fn build_tree(flat: Vec<ArchiveEntry>) -> Vec<ArchiveEntry> {
    let mut root: Vec<ArchiveEntry> = Vec::new();

    for entry in flat {
        let path = match &entry.path {
            Some(p) => p.clone(),
            None => {
                root.push(entry);
                continue;
            }
        };

        let components: Vec<Component> = path.components().collect();
        insert_entry(&mut root, &components, entry);
    }

    root
}

fn insert_entry(children: &mut Vec<ArchiveEntry>, components: &[Component], entry: ArchiveEntry) {
    // Case 1: there is an implicit directory node that has the correct children
    if components.len() == 1 {
        let name = components[0].as_os_str();

        if let EntryData::Directory(_) = &entry.data
            && children
                .iter()
                .any(|e| e.path.as_ref().and_then(|p| p.file_name()) == Some(name))
        {
            return;
        }

        children.push(entry);
        return;
    }

    // Case 2: the parent directory node is among the current cildren
    let parent_name = components[0].as_os_str();
    if let Some(parent) = children.iter_mut().find(|e| {
        e.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n == parent_name)
            .unwrap_or(false)
    }) {
        if let EntryData::Directory(ref mut sub) = parent.data {
            insert_entry(sub, &components[1..], entry);
        }
        return;
    }

    // Case 3: the parent directory node doesn't exist yet; recurse into it
    let parent_path = PathBuf::from(parent_name);
    let mut implicit_dir = ArchiveEntry {
        path: Some(parent_path),
        data: EntryData::Directory(Vec::new()),
    };

    if let EntryData::Directory(ref mut sub) = implicit_dir.data {
        insert_entry(sub, &components[1..], entry);
    }

    children.push(implicit_dir);
}
