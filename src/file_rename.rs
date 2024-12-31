use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::path_checkout::MetadataCollection;

const GUID: &str = "2A96D0978ECC9709298A";

#[derive(Debug)]
pub struct PrepareName {
    pub original_path: PathBuf,
    pub new_path: PathBuf,
    pub pre_path: PathBuf,
}
impl Default for PrepareName {
    fn default() -> Self {
        Self {
            original_path: PathBuf::new(),
            new_path: PathBuf::new(),
            pre_path: PathBuf::new(),
        }
    }
}

#[derive(Debug)]
pub struct FileInfos {
    pub is_exist: bool,
    pub is_file: bool,
    pub packed_info: MetadataCollection,
    pub exchange: PrepareName,
}
impl Default for FileInfos {
    fn default() -> Self {
        Self {
            is_exist: false,
            is_file: false,
            packed_info: MetadataCollection {
                ..Default::default()
            },
            exchange: PrepareName {
                ..Default::default()
            },
        }
    }
}

pub struct NameExchange {
    pub f1: FileInfos,
    pub f2: FileInfos,
}
/// 改名逻辑主体整合
impl NameExchange {
    ///用于初始化储存所有信息的结构体
    pub fn new() -> NameExchange {
        NameExchange {
            f1: FileInfos {
                ..Default::default()
            },
            f2: FileInfos {
                ..Default::default()
            },
        }
    }

    ///获取临时文件名与改后文件名
    pub fn make_name(dir: &Path, other_name: &String, ext: &String) -> (PathBuf, PathBuf) {
        let mut dir = dir.to_path_buf();
        let ext = ext.to_string();
        let mut other_name = other_name.to_string();
        let mut new_name = dir.to_path_buf(); //C:/    (a)

        //任意长字符串用作区分
        let mut temp_additional_name = GUID.to_string();
        temp_additional_name.push_str(&ext); //AAAAA.txt
        dir.push(&temp_additional_name); //C:/AAAAA.txt    (b)
        let new_pre_name = dir.to_path_buf();

        other_name.push_str(&ext); //AnotherFileName.txt
        new_name.push(&other_name); //C:/AnotherFileName.txt    (a)

        (new_pre_name, new_name)
    }

    ///改名具体执行部分
    pub fn rename_each(&self, is_nested: bool, file1_first: bool) -> i32 {
        let mut path1 = self.f2.exchange.original_path.clone();
        let mut final_name1 = self.f2.exchange.new_path.clone();
        let mut path2 = self.f1.exchange.original_path.clone();
        let mut final_name2 = self.f1.exchange.new_path.clone();
        let mut tmp_name2 = self.f1.exchange.pre_path.clone();
        if file1_first {
            path1 = self.f1.exchange.original_path.clone();
            final_name1 = self.f1.exchange.new_path.clone();
            path2 = self.f2.exchange.original_path.clone();
            final_name2 = self.f2.exchange.new_path.clone();
            tmp_name2 = self.f2.exchange.pre_path.clone();
        }

        let get_err_or_ok = |x: io::Result<()>| -> i32 {
            match x {
                Ok(_) => 0,
                Err(x) => match x.kind() {
                    io::ErrorKind::PermissionDenied => return 2_i32,
                    io::ErrorKind::AlreadyExists => return 3_i32,
                    _ => return 255_i32,
                },
            }
        };
        //1 first
        if is_nested {
            //如果存在相关性（父子目录或文件），后面的exit(3)是为了核验是否有权限改名
            let rename_1_result = get_err_or_ok(fs::rename(&path1, &final_name1));
            let rename_2_result = get_err_or_ok(fs::rename(&path2, &final_name2));
            if rename_1_result != 0 {
                println!("FAILED: \n{:?} => {:?}", &path1, &final_name1);
                return rename_1_result;
            } else if rename_2_result != 0 {
                println!("FAILED: \n{:?} => {:?}", &path2, &final_name2);
                return rename_2_result;
            } else {
                println!("SUCCESS: \n{:?} => {:?}", &path1, &final_name1);
                println!("SUCCESS: \n{:?} => {:?}", &path2, &final_name2);
                return 0;
            }
        } else {
            //不存在相关性：正常操作
            let rename_1_result = get_err_or_ok(fs::rename(&path2, &tmp_name2));
            let rename_2_result = get_err_or_ok(fs::rename(&path1, &final_name1));
            let rename_3_result = get_err_or_ok(fs::rename(&tmp_name2, &final_name2));
            if rename_1_result != 0 {
                println!("FAILED: \n{:?} => {:?}", &path2, &tmp_name2);
                return rename_1_result;
            } else if rename_2_result != 0 {
                println!("FAILED: \n{:?} => {:?}", &path1, &final_name1);
                return rename_2_result;
            } else if rename_3_result != 0 {
                println!("FAILED: \n{:?} => {:?}", &tmp_name2, &final_name2);
                return rename_3_result;
            } else {
                println!("SUCCESS: \n{:?} => {:?}", &path2, &tmp_name2);
                println!("SUCCESS: \n{:?} => {:?}", &path1, &final_name1);
                println!("SUCCESS: \n{:?} => {:?}", &tmp_name2, &final_name2);
                return 0;
            }
        }
    }
}
