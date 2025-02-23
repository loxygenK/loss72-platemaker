use std::{
    any::type_name,
    path::{Path, PathBuf},
};

use loss72_platemaker_core::{log, util::get_slice_by_char};

#[derive(Debug)]
pub struct Construction<'c> {
    pub dir: &'c Path,
    pub content: Vec<ConstructFile<'c>>,
    pub sub_dir: Vec<Construction<'c>>,
}

pub struct ConstructFile<'c> {
    pub path: &'c Path,
    pub content: &'c str,
}

impl std::fmt::Debug for ConstructFile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<Self>().rsplit("::").next().unwrap())
            .field("path", &self.path)
            .field(
                "content",
                &format_args!(
                    "\"{} ... (truncated)\"",
                    get_slice_by_char(self.content, 0..40).escape_debug()
                ),
            )
            .finish()
    }
}

#[derive(Debug)]
pub struct ConstructionPlan<'p> {
    pub dirs: Vec<PathBuf>,
    pub files: Vec<(PathBuf, &'p str)>,
}

impl ConstructionPlan<'_> {
    pub fn merge(&mut self, other: Self) {
        self.dirs.extend(other.dirs);
        self.files.extend(other.files);
    }

    pub fn prefix_dirs(&mut self, prefix: &Path) {
        self.dirs
            .iter_mut()
            .for_each(|mut dir| *dir = prefix.join(&mut dir));
        self.files
            .iter_mut()
            .for_each(|files| *files = (prefix.join(&mut files.0), files.1));
    }
}

impl Construction<'_> {
    pub fn plan(&self, root: &Path) -> ConstructionPlan {
        let mut plan = self._plan(root);

        plan.dirs.sort_by_key(|x| x.as_os_str().len());

        plan
    }

    fn _plan(&self, parent: &Path) -> ConstructionPlan {
        let root = parent.join(self.dir);

        let mut plan = ConstructionPlan {
            dirs: vec![root.clone()],
            files: self
                .content
                .iter()
                .map(|plan| (root.join(plan.path), plan.content))
                .collect::<Vec<_>>(),
        };

        let mut used_path = self
            .content
            .iter()
            .filter_map(|file| file.path.parent())
            .map(|path| root.join(path))
            .collect::<Vec<_>>();

        used_path.sort();
        used_path.dedup();

        plan.dirs.extend(used_path);

        for sub_dir in &self.sub_dir {
            let mut sub_plan = sub_dir._plan(&root);
            sub_plan.prefix_dirs(self.dir);

            plan.merge(sub_plan);
        }

        plan
    }
}

impl ConstructionPlan<'_> {
    pub fn execute(&self) -> Result<(), std::io::Error> {
        for dir in self.dirs.iter().rev() {
            log!(step: "Creating dir {}", dir.display());
            std::fs::create_dir_all(dir)?;
        }

        for file in self.files.iter() {
            log!(step: "Writing file {}", file.0.display());
            std::fs::write(&file.0, file.1)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{ConstructFile, Construction};

    #[cfg(test)]
    #[test]
    pub fn flattenable() {
        let construction = Construction {
            dir: Path::new("pages"),
            content: vec![
                ConstructFile {
                    path: Path::new("1.html"),
                    content: "AAA",
                },
                ConstructFile {
                    path: Path::new("2.html"),
                    content: "AAA",
                },
                ConstructFile {
                    path: Path::new("3.html"),
                    content: "AAA",
                },
            ],
            sub_dir: vec![Construction {
                dir: Path::new("sub-1"),
                content: vec![
                    ConstructFile {
                        path: Path::new("1.html"),
                        content: "A",
                    },
                    ConstructFile {
                        path: Path::new("2.html"),
                        content: "B",
                    },
                    ConstructFile {
                        path: Path::new("3.html"),
                        content: "C",
                    },
                ],
                sub_dir: vec![],
            }],
        };

        panic!("{:#?}", construction.plan(Path::new("/root")));
    }
}
