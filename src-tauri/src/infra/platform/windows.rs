/// Windows 平台特定实现
use crate::domain::identity::MachineIds;
use crate::error::AppError;
use super::PlatformOps;

pub struct WindowsOps;

impl PlatformOps for WindowsOps {
    fn update_system_ids(&self, ids: &MachineIds) -> Result<(), AppError> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        if let Ok(key) = hklm.open_subkey_with_flags(
            "SOFTWARE\\Microsoft\\Cryptography",
            KEY_SET_VALUE,
        ) {
            let _ = key.set_value("MachineGuid", &ids.machine_id);
        }

        if let Ok(key) = hklm.open_subkey_with_flags(
            "SOFTWARE\\Microsoft\\SQMClient",
            KEY_SET_VALUE,
        ) {
            let _ = key.set_value("MachineId", &ids.sqm_id);
        }

        Ok(())
    }

    fn is_admin(&self) -> bool {
        use windows::Win32::Security::{
            GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
        };
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        unsafe {
            let process = GetCurrentProcess();
            let mut token_handle = std::mem::zeroed();
            if OpenProcessToken(process, TOKEN_QUERY, &mut token_handle).is_err() {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut return_length = 0u32;

            if GetTokenInformation(
                token_handle,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            )
            .is_err()
            {
                return false;
            }

            elevation.TokenIsElevated != 0
        }
    }
}
