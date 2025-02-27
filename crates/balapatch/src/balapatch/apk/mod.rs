pub mod apktool;
pub mod zipalign;

// pub fn install_apk(server: &mut ADBServer, apk_path: &str) -> anyhow::Result<(), RustADBError> {
//     let mut device = server.get_device()?;
//     let apk_path = Path::new(apk_path);
// 
//     if !apk_path.exists() {
//         return Err(RustADBError::ADBRequestFailed("fuck".to_string()));
//     } else {
//         device.install(apk_path)?;
//     }
// 
//     Ok(())
// }