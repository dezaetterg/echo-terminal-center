use std::ffi::CStr;
use std::mem::MaybeUninit;

pub fn get_kernel() -> String {
    unsafe {
        let mut uts = MaybeUninit::<libc::utsname>::uninit();
        if libc::uname(uts.as_mut_ptr()) == 0 {
            let uts = uts.assume_init();
            let sysname = CStr::from_ptr(uts.sysname.as_ptr())
                .to_string_lossy()
                .into_owned();
            let release = CStr::from_ptr(uts.release.as_ptr())
                .to_string_lossy()
                .into_owned();
            if !sysname.is_empty() && !release.is_empty() {
                return format!("{sysname} {release}");
            }
        }
    }

    if let Ok(release) = std::fs::read_to_string("/proc/sys/kernel/osrelease") {
        let release = release.trim();
        if !release.is_empty() {
            return format!("Linux {release}");
        }
    }

    "—".to_string()
}
