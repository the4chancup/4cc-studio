//! The `overrides/` tree (`team_compiler/pipeline.md` "5. Writer", item 1): files the operator
//! puts in the data directory's `overrides/` folder go into the CPK as they are, each at its path
//! below that folder, over whatever an export or the bins would put there.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use studio_core::{Disposition, Message, Scope};

use crate::messages::{Code, tool_message};

/// The folder's name in the data directory.
const FOLDER_NAME: &str = "overrides";

/// Every file below `<data_dir>/overrides/`, by CPK path (its path below the folder,
/// `/`-separated) with where it is on disk, and the `overrides_active` note reporting them. A
/// `BTreeMap` keeps the paths in byte order, so the CPK lays the overrides out the same on every
/// file system. Nothing, and no note, without a data directory, without the folder, or with no
/// file in it. A folder that cannot be listed, or a name that is not UTF-8, is an error naming
/// the path.
pub(crate) fn list(
    data_dir: Option<&Path>,
) -> anyhow::Result<(BTreeMap<String, PathBuf>, Option<Message>)> {
    let mut files = BTreeMap::new();
    let Some(data_dir) = data_dir else {
        return Ok((files, None));
    };
    let folder = data_dir.join(FOLDER_NAME);
    if !folder.is_dir() {
        return Ok((files, None));
    }
    walk(&folder, "", &mut files)?;
    if files.is_empty() {
        return Ok((files, None));
    }
    let message = tool_message(
        Code::OverridesActive,
        Scope::Run,
        Disposition::Keep,
        vec![
            ("folder", folder.display().to_string()),
            ("files", files.len().to_string()),
        ],
    );
    Ok((files, Some(message)))
}

/// Lists the files of `folder` (at the CPK path `prefix`) and everything below it into `files`.
fn walk(folder: &Path, prefix: &str, files: &mut BTreeMap<String, PathBuf>) -> anyhow::Result<()> {
    let cannot_read = || format!("{}: cannot read the overrides folder", folder.display());
    for entry in fs::read_dir(folder).with_context(cannot_read)? {
        let entry = entry.with_context(cannot_read)?;
        let path = entry.path();
        let name = entry.file_name();
        // A CPK path is text: a name that is not cannot be put in the CPK as it is spelled.
        let Some(name) = name.to_str() else {
            bail!("{}: the name is not valid UTF-8", path.display());
        };
        let cpk_path = if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{prefix}/{name}")
        };
        if path.is_dir() {
            walk(&path, &cpk_path, files)?;
        } else {
            files.insert(cpk_path, path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use studio_core::Severity;

    use super::*;
    use crate::testing::scratch;

    #[test]
    fn every_file_below_the_folder_is_listed_by_its_cpk_path_in_path_order() {
        let temp = scratch("overrides_listed");
        let data_dir = temp.path();
        let folder = data_dir.join("overrides");
        for path in ["common/etc/TeamColor.bin", "Asset/b.bin", "Asset/a/c.bin"] {
            let file = folder.join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, path).unwrap();
        }

        let (files, message) = list(Some(data_dir)).unwrap();

        let listed: Vec<(&str, PathBuf)> = files
            .iter()
            .map(|(path, file)| (path.as_str(), file.clone()))
            .collect();
        assert_eq!(
            listed,
            [
                (
                    "Asset/a/c.bin",
                    folder.join("Asset").join("a").join("c.bin")
                ),
                ("Asset/b.bin", folder.join("Asset").join("b.bin")),
                (
                    "common/etc/TeamColor.bin",
                    folder.join("common").join("etc").join("TeamColor.bin")
                ),
            ]
        );
        let message = message.expect("a tree holding files is reported");
        assert_eq!(message.code.code, "overrides_active");
        assert_eq!(
            (message.severity, message.disposition, &message.scope),
            (Severity::Info, Disposition::Keep, &Scope::Run)
        );
        assert_eq!(
            message.context,
            [
                ("folder".to_owned(), folder.display().to_string()),
                ("files".to_owned(), "3".to_owned())
            ]
        );
    }

    #[test]
    fn no_data_directory_no_folder_or_an_empty_one_lists_nothing() {
        let temp = scratch("overrides_none");
        let data_dir = temp.path();
        assert_eq!(list(None).unwrap(), (BTreeMap::new(), None));
        assert_eq!(list(Some(data_dir)).unwrap(), (BTreeMap::new(), None));
        fs::create_dir(data_dir.join("overrides")).unwrap();
        assert_eq!(list(Some(data_dir)).unwrap(), (BTreeMap::new(), None));
        // A folder holding only folders holds no file.
        fs::create_dir_all(data_dir.join("overrides/common/etc")).unwrap();
        assert_eq!(list(Some(data_dir)).unwrap(), (BTreeMap::new(), None));
    }
}
