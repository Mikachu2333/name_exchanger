use std::{
    ffi::{c_char, CStr},
    path::PathBuf,
};

use file_rename::NameExchange;
use path_checkout::GetPathInfo;
mod file_rename;
mod path_checkout;

#[no_mangle]
/// # Safety
/// 最终暴露的执行函数，传入两个路径String，返回一个u8
///
/// 0 => Success，1 => No Exist
///
/// 2 => Permission Denied，3 => New File Already Exists
///
/// 255 => UNKNOWN ERROR
pub extern "C" fn exchange(path1: *const c_char, path2: *const c_char) -> i32 {
    let binding = std::env::current_exe().unwrap();
    let exe_dir = binding.parent().unwrap();

    let transformer = |s: *const c_char| unsafe { CStr::from_ptr(s) }.to_string_lossy().to_string();

    let path1 = transformer(path1);
    let path2 = transformer(path2);

    let mut all_infos = NameExchange::new();

    // 用于校验文件夹路径最后是否为斜杠与双引号的闭包
    let dir_check = |s: String| PathBuf::from(s.trim().trim_matches(['"', '\\', '\'']));
    let mut packed_path = GetPathInfo {
        path1: dir_check(path1),
        path2: dir_check(path2),
    };

    (all_infos.f1.is_exist, all_infos.f2.is_exist) = (packed_path).if_exist(exe_dir);
    if (!all_infos.f1.is_exist) || (!all_infos.f2.is_exist) {
        return 1_i32;
    }
    if packed_path.path1 == packed_path.path2 {
        return 2_i32;
    }
    all_infos.f1.exchange.original_path = packed_path.path1.clone();
    all_infos.f2.exchange.original_path = packed_path.path2.clone();

    (all_infos.f1.is_file, all_infos.f2.is_file) = packed_path.if_file();

    (all_infos.f1.packed_info, all_infos.f2.packed_info) =
        packed_path.metadata_collect(all_infos.f1.is_file, all_infos.f2.is_file);

    (
        all_infos.f1.exchange.pre_path,
        all_infos.f1.exchange.new_path,
    ) = NameExchange::make_name(
        &all_infos.f1.packed_info.parent_dir,
        &all_infos.f2.packed_info.name,
        &all_infos.f1.packed_info.ext,
    );
    (
        all_infos.f2.exchange.pre_path,
        all_infos.f2.exchange.new_path,
    ) = NameExchange::make_name(
        &all_infos.f2.packed_info.parent_dir,
        &all_infos.f1.packed_info.name,
        &all_infos.f2.packed_info.ext,
    );

    let mut packed_path_new = GetPathInfo {
        path1: all_infos.f1.exchange.new_path.clone(),
        path2: all_infos.f2.exchange.new_path.clone(),
    };
    let (exist_new_1, exist_new_2) = GetPathInfo::if_exist(&mut packed_path_new, exe_dir);
    let same_dir = GetPathInfo::if_same_dir(&packed_path_new);
    if !same_dir && (exist_new_1 || exist_new_2) {
        //不能因为rename函数里面有就删了……
        /*
        println!(
            "same:{}\tnew1:{}\tnew2:{}",
            same_dir, exist_new_1, exist_new_2
        );
        */
        return 3_i32;
    }

    //1 -> file1 should be renamed first
    let mode = packed_path.if_root();

    /*
    println!(//test
        "f1: {}\t{}\t{}",
        all_infos.f1.packed_info.parent_dir.display(),
        all_infos.f1.packed_info.name,
        all_infos.f1.packed_info.ext
    );
    println!(
        "f2: {}\t{}\t{}",
        all_infos.f2.packed_info.parent_dir.display(),
        all_infos.f2.packed_info.name,
        all_infos.f2.packed_info.ext
    );
    */
    
    if all_infos.f1.is_file & all_infos.f2.is_file {
        //all files
        NameExchange::rename_each(&all_infos, false, true)
    } else if (!all_infos.f1.is_file) && (!all_infos.f2.is_file) {
        //all dirs
        if mode == 1 {
            //file1 contains file2
            NameExchange::rename_each(&all_infos, true, true)
        } else if mode == 2 {
            //file2 contains file1
            NameExchange::rename_each(&all_infos, true, false)
        } else {
            //no contains
            NameExchange::rename_each(&all_infos, false, true)
        }
    } else {
        // one file and one dir
        if all_infos.f1.is_file {
            //1 is file and 2 is dir so impossible 1 contains 2
            if mode == 1 {
                //file1 rename first
                NameExchange::rename_each(&all_infos, true, true)
            } else {
                NameExchange::rename_each(&all_infos, false, true)
            }
        } else {
            //same
            if mode == 2 {
                //file2 rename first
                NameExchange::rename_each(&all_infos, true, false)
            } else {
                //file2 rename first
                NameExchange::rename_each(&all_infos, false, false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env::args;
    use std::{ffi::CString, fs::remove_file};

    fn clear_olds() {
        let _ = remove_file("file1.ext1");
        let _ = remove_file("file2.ext2");
        let _ = remove_file("file2.ext1");
        let _ = remove_file("file1.ext2");

        let mut new_file1 = std::fs::File::create("file1.ext1").unwrap();
        let mut new_file2 = std::fs::File::create("file2.ext2").unwrap();
        let _ = std::io::Write::write_all(&mut new_file1, b"");
        let _ = std::io::Write::write_all(&mut new_file2, b"");
    }
    #[test]
    fn it_works() {
        clear_olds();
        // 0 => Success，1 => No Exist
        // 2 => Permission Denied，3 => New File Already Exists

        let trans = |s: String| CString::new(s).unwrap();
        let _test_path1 = trans(r"file1.ext1".to_owned());
        let _test_path2 = trans(r"file1.ext1".to_owned());

        let mut a: Vec<CString> = args().map(|f| trans(f)).collect();
        a.remove(0);

        let run_result = super::exchange(a[1].as_ptr(), a[2].as_ptr());
        println!("{}", run_result);
    }
}
