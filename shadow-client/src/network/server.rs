use crate::misc;
use crabgrab::{
    capturable_content::{CapturableContent, CapturableContentFilter},
    capture_stream::{CaptureConfig, CaptureStream},
    feature::screenshot,
};
use remoc::{codec, prelude::*};
use shadow_common::{
    client::{self as sc, SystemPowerAction},
    error::ShadowError,
    server as ss,
};
use std::{path::Path, sync::Arc};
use sysinfo::System;
use tokio::{fs, sync::RwLock};

#[allow(dead_code)]
#[derive(Debug)]
pub struct ClientCfg {
    version: String,
}

impl Default for ClientCfg {
    fn default() -> Self {
        let version = misc::get_version();

        Self { version }
    }
}

pub struct ClientObj {
    cfg: ClientCfg,
    pub client: Option<Arc<RwLock<ss::ServerClient<codec::Bincode>>>>,
}

impl ClientObj {
    pub fn new() -> Self {
        Self {
            cfg: ClientCfg::default(),
            client: None,
        }
    }
}

#[rtc::async_trait]
impl sc::Client for ClientObj {
    async fn handshake(&self) -> Result<sc::Handshake, ShadowError> {
        Ok(sc::Handshake {
            message: format!("{:#?}", self.cfg),
        })
    }

    async fn get_system_info(&self) -> Result<sc::SystemInfo, ShadowError> {
        let mut system = System::new_all();
        system.refresh_all();

        let system_name = System::name().unwrap_or_default();
        let kernel_version = System::kernel_version().unwrap_or_default();
        let os_version = System::os_version().unwrap_or_default();
        let host_name = System::host_name().unwrap_or_default();

        Ok(sc::SystemInfo {
            system_name,
            kernel_version,
            os_version,
            host_name,
        })
    }

    async fn system_power(&self, action: SystemPowerAction) -> Result<bool, ShadowError> {
        match match action {
            SystemPowerAction::Shutdown => system_shutdown::shutdown(),
            SystemPowerAction::Reboot => system_shutdown::reboot(),
            SystemPowerAction::Logout => system_shutdown::logout(),
            SystemPowerAction::Sleep => system_shutdown::sleep(),
            SystemPowerAction::Hibernate => system_shutdown::hibernate(),
        } {
            Ok(_) => Ok(true),
            Err(_) => Err(ShadowError::SystemPowerError),
        }
    }

    async fn get_installed_apps(&self) -> Result<Vec<sc::App>, ShadowError> {
        match installed::list() {
            Ok(l) => Ok(l
                .filter(|app| app.name().to_string().is_empty() == false)
                .map(|app| sc::App {
                    name: app.name().to_string(),
                    publisher: app.publisher().to_string(),
                    version: app.version().to_string(),
                })
                .collect()),
            Err(e) => Err(ShadowError::QueryAppsError(e.to_string())),
        }
    }

    async fn get_processes(&self) -> Result<Vec<sc::Process>, ShadowError> {
        let mut system = System::new();
        system.refresh_processes();

        Ok(system
            .processes()
            .iter()
            .map(|(pid, process)| {
                let parent_pid = match process.parent() {
                    Some(p) => Some(p.as_u32()),
                    None => None,
                };

                sc::Process {
                    pid: pid.as_u32(),
                    name: process.name().into(),
                    parent_pid,
                    exe: process
                        .exe()
                        .unwrap_or(Path::new(""))
                        .to_string_lossy()
                        .to_string(),
                    start_time: process.start_time(),
                    cwd: process
                        .cwd()
                        .unwrap_or(Path::new(""))
                        .to_string_lossy()
                        .to_string(),
                }
            })
            .collect())
    }

    async fn get_file_list(&self, dir: String) -> Result<Vec<sc::File>, ShadowError> {
        let mut ret = Vec::new();
        let mut list = match fs::read_dir(&dir).await {
            Ok(l) => l,
            Err(e) => return Err(ShadowError::QueryFilesError(dir, e.to_string())),
        };

        while let Some(f) = match list.next_entry().await {
            Ok(e) => e,
            Err(e) => return Err(ShadowError::QueryFilesError(dir, e.to_string())),
        } {
            let name = match f.file_name().into_string() {
                Ok(n) => n,
                Err(_) => continue,
            };
            let file_type = match f.file_type().await {
                Ok(t) => t,
                Err(_) => continue,
            };

            ret.push(sc::File {
                name,
                is_dir: file_type.is_dir(),
            });
        }

        Ok(ret)
    }

    async fn get_displays(&self) -> Result<Vec<sc::Display>, ShadowError> {
        // let token = match CaptureStream::test_access(false) {
        //     Some(t) => t,
        //     None => match CaptureStream::request_access(false).await {
        //         Some(t) => t,
        //         None => return Err(ShadowError::AccessDenied),
        //     },
        // };

        let content = CapturableContent::new(CapturableContentFilter::EVERYTHING).await?;
        Ok(content
            .displays()
            .map(|d| sc::Display { rect: d.rect() })
            .collect())
    }
}
