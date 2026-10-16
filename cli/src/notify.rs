//! The Windows notification that says a download is finished. Clicking it
//! opens the folder.

use std::path::Path;

/// Shows the notification. `false` when Windows did not take it, so the
/// console has to say it instead.
#[cfg(windows)]
pub fn saved(file: &Path) -> bool {
    windows_toast::show(file).is_ok()
}

#[cfg(not(windows))]
pub fn saved(_file: &Path) -> bool {
    false
}

#[cfg(windows)]
mod windows_toast {
    use std::path::Path;
    use windows::Data::Xml::Dom::XmlDocument;
    use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};
    use windows::Win32::System::Registry::{HKEY_CURRENT_USER, REG_SZ, RegSetKeyValueW};
    use windows::core::{HSTRING, Result, w};

    /// The name Windows knows keepclip by in the notification center.
    const APP_ID: &str = "keepclip";

    pub fn show(file: &Path) -> Result<()> {
        register()?;
        let name = file
            .file_name()
            .map_or(String::new(), |n| n.to_string_lossy().into_owned());
        let folder = file.parent().unwrap_or(file);
        let xml = format!(
            "<toast activationType=\"protocol\" launch=\"{}\">\
             <visual><binding template=\"ToastGeneric\">\
             <text>Download finished</text><text>{}</text>\
             <text placement=\"attribution\">Click to open the folder</text>\
             </binding></visual></toast>",
            escape(&file_uri(folder)),
            escape(&name)
        );
        let document = XmlDocument::new()?;
        document.LoadXml(&HSTRING::from(xml))?;
        let toast = ToastNotification::CreateToastNotification(&document)?;
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(APP_ID))?.Show(&toast)
    }

    /// An app without an installer is unknown to the notification center
    /// until its id has a name under `HKEY_CURRENT_USER`.
    fn register() -> Result<()> {
        let name: Vec<u16> = "keepclip".encode_utf16().chain([0]).collect();
        // SAFETY: the data points to `name`, which outlives the call, and the
        // length counts its bytes, the final zero included.
        unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                w!("Software\\Classes\\AppUserModelId\\keepclip"),
                w!("DisplayName"),
                REG_SZ.0,
                Some(name.as_ptr().cast()),
                (name.len() * 2) as u32,
            )
        }
        .ok()
    }

    /// `file:///C:/Users/you/My%20Videos`, for the click on the notification.
    fn file_uri(path: &Path) -> String {
        let text = path.to_string_lossy().replace('\\', "/");
        let (mut uri, rest) = match text.strip_prefix("//") {
            Some(network) => ("file://".to_string(), network.to_string()),
            None => ("file:///".to_string(), text),
        };
        for byte in rest.bytes() {
            if byte.is_ascii_alphanumeric() || b"-._~/:".contains(&byte) {
                uri.push(byte as char);
            } else {
                uri.push_str(&format!("%{byte:02X}"));
            }
        }
        uri
    }

    fn escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn folder_links_are_file_uris() {
            assert_eq!(
                file_uri(Path::new("C:\\Users\\you\\My Videos")),
                "file:///C:/Users/you/My%20Videos"
            );
            assert_eq!(
                file_uri(Path::new("\\\\server\\share\\clips")),
                "file://server/share/clips"
            );
            assert_eq!(file_uri(Path::new("D:\\Vídeos")), "file:///D:/V%C3%ADdeos");
        }
    }
}
