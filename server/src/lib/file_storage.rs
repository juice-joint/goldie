use std::path::PathBuf;

/// The folder where `eframe` will store its state.
///
/// The given `app_id` is either the
/// [`egui::ViewportBuilder::app_id`] of [`crate::NativeOptions::viewport`]
/// or the title argument to [`crate::run_native`].
///
/// On native, the path is:
/// * Linux:   `/home/UserName/.local/share/APP_ID`
/// * macOS:   `/Users/UserName/Library/Application Support/APP_ID`
/// * Windows: `C:\Users\UserName\AppData\Roaming\APP_ID\data`
pub fn storage_dir(app_id: &str) -> Option<PathBuf> {
    use crate::lib::os::OperatingSystem as OS;
    use std::env::var_os;
    match OS::from_target_os() {
        OS::Nix => var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| home::home_dir().map(|p| p.join(".local").join("share")))
            .map(|p| {
                p.join(
                    app_id
                        .to_lowercase()
                        .replace(|c: char| c.is_ascii_whitespace(), ""),
                )
            }),
        OS::Mac => home::home_dir().map(|p| {
            p.join("Library")
                .join("Application Support")
                .join(app_id.replace(|c: char| c.is_ascii_whitespace(), "-"))
        }),
        OS::Windows => roaming_appdata().map(|p| p.join(app_id).join("data")),
        OS::Unknown | OS::Android | OS::IOS => None,
    }
}

// Adapted from
// https://github.com/rust-lang/cargo/blob/6e11c77384989726bb4f412a0e23b59c27222c34/crates/home/src/windows.rs#L19-L37
#[cfg(all(windows, not(target_vendor = "uwp")))]
#[allow(unsafe_code)]
fn roaming_appdata() -> Option<PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::ptr;
    use std::slice;

    use windows_sys::Win32::Foundation::S_OK;
    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::UI::Shell::{
        FOLDERID_RoamingAppData, SHGetKnownFolderPath, KF_FLAG_DONT_VERIFY,
    };

    extern "C" {
        fn wcslen(buf: *const u16) -> usize;
    }
    unsafe {
        let mut path = ptr::null_mut();
        match SHGetKnownFolderPath(
            &FOLDERID_RoamingAppData,
            KF_FLAG_DONT_VERIFY as u32,
            std::ptr::null_mut(),
            &mut path,
        ) {
            S_OK => {
                let path_slice = slice::from_raw_parts(path, wcslen(path));
                let s = OsString::from_wide(&path_slice);
                CoTaskMemFree(path.cast());
                Some(PathBuf::from(s))
            }
            _ => {
                // Free any allocated memory even on failure. A null ptr is a no-op for `CoTaskMemFree`.
                CoTaskMemFree(path.cast());
                None
            }
        }
    }
}

#[cfg(any(not(windows), target_vendor = "uwp"))]
fn roaming_appdata() -> Option<PathBuf> {
    None
}
