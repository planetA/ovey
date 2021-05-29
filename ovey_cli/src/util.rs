use std::fs;
use regex::Regex;

/// The directory on Linux where ib devices like "ovey0" or "rxe0" show up as files.
pub const INFINIBAND_SYSFS_DEVICES_PATH: &str = "/sys/class/infiniband/";

fn build_ovey_regex() -> Regex {
    Regex::new(
        libocp::ocp_properties::DEVICE_NAME_PATTERN
    ).unwrap()
}

/// Returns a list of all ovey devices registered by the kernel.
/// Instead of doing additional (and more error-prone + complex)
/// logic by asking libibverbs or Kernel via OCP, we just read
/// the local sys-directory. See [`INFINIBAND_SYSFS_DEVICES_PATH`]
pub fn get_all_local_ovey_devices() -> Vec<String> {
    let mut devs = vec![];
    let regex = build_ovey_regex();

    let readdir = fs::read_dir(INFINIBAND_SYSFS_DEVICES_PATH);
    if readdir.is_err() { return devs }
    let readdir = readdir.unwrap();

    for dir in readdir {
        if let std::io::Result::Ok(entry) = dir {
            let dev_name = entry.file_name().to_string_lossy().into_owned();
            if regex.is_match(&dev_name) {
                devs.push(dev_name)
            }
        }
    }

    devs
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_local_ovey_devices() {
        let devs = get_all_local_ovey_devices();
        println!("{:#?}", devs);
    }
}
