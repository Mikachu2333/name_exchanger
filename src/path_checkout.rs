use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct MetadataCollection {
    pub name: String,
    pub ext: String,
    pub parent_dir: PathBuf,
}
impl Default for MetadataCollection {
    fn default() -> Self {
        Self {
            name: "".to_owned(),
            ext: "".to_owned(),
            parent_dir: PathBuf::new(),
        }
    }
}

#[derive(Debug)]
pub struct GetPathInfo {
    pub path1: PathBuf,
    pub path2: PathBuf,
}
/// 所有路径相关的操作
impl GetPathInfo {
    /// 校验路径是否存在；如果是相对路径，尝试转化为绝对路径
    pub fn if_exist(&mut self, dir: &Path) -> (bool, bool) {
        if self.path1.is_relative() {
            self.path1 = dir.join(self.path1.clone().file_name().unwrap());
        }
        if self.path2.is_relative() {
            self.path2 = dir.join(self.path2.clone().file_name().unwrap());
        }
        /*
        println!(
            "Path1: {}\tPath2: {}",
            self.path1.display(),
            self.path2.display()
        ); //test
        */
        (self.path1.exists(), self.path2.exists())
    }

    ///输入的文件类型是否为文件夹
    pub fn if_file(&self) -> (bool, bool) {
        (self.path1.is_file(), self.path2.is_file())
    }

    pub fn if_same_dir(&self) -> bool {
        self.path1.parent().unwrap() == self.path2.parent().unwrap()
    }

    ///检测是否存在包含关系（父子目录问题）
    pub fn if_root(&self) -> u8 {
        //下面必须统一取小写或大写，因为rust的“contains()”大小写敏感
        let path1 = self.path1.to_string_lossy().to_ascii_lowercase();
        let path2 = self.path2.to_string_lossy().to_ascii_lowercase();

        if path1.contains(&path2) {
            //path1 should be renamed first
            1
        } else if path2.contains(&path1) {
            //path2 should be renamed first
            2
        } else {
            //no-influence
            0
        }
    }

    ///获取文件名称（无后缀）、后缀、所在文件夹（父文件夹）
    fn get_info(file_path: &Path, is_file: bool) -> MetadataCollection {
        //取文件名（String）格式，如果在取后缀则加“.”，此处还兼有unwrap的功能
        let get_string_closure = |original_result: &Option<&OsStr>, is_ext: bool| {
            match original_result {
                Some(i) => {
                    if is_ext {
                        //是否在计算后缀，如果不是，去掉一开始的“.”
                        ".".to_owned() + i.to_str().unwrap()
                    } else {
                        i.to_str().unwrap().to_string()
                    }
                }
                /*
                取不到就无视
                因前面已经核验完毕，所以此处如果出现Err则是特殊文件命名所致，不影响后面所有操作。
                e.g. "C:\\.cargo\\.config"，该文件取不到后缀，该文件夹也取不到后缀
                */
                None => String::new(),
            }
        };

        if !is_file {
            return MetadataCollection {
                name: {
                    get_string_closure(&file_path.file_stem(), false)
                        + get_string_closure(&file_path.extension(), true).as_ref()
                },
                ext: String::new(),
                parent_dir: {
                    match &file_path.parent() {
                        Some(i) => i.to_path_buf(),
                        None => PathBuf::new(),
                    }
                },
            };
        } else {
            return MetadataCollection {
                name: get_string_closure(&file_path.file_stem(), false),
                ext: get_string_closure(&file_path.extension(), true),
                parent_dir: {
                    match &file_path.parent() {
                        Some(i) => i.to_path_buf(),
                        None => PathBuf::new(),
                    }
                },
            };
        }
    }

    pub fn metadata_collect(
        &self,
        is_file1: bool,
        is_file2: bool,
    ) -> (MetadataCollection, MetadataCollection) {
        let metadata1 = GetPathInfo::get_info(&self.path1, is_file1);
        let metadata2 = GetPathInfo::get_info(&self.path2, is_file2);
        (metadata1, metadata2)
    }
}
