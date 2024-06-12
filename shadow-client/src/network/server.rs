use crate::misc;
use crabgrab::{
    capturable_content::{CapturableContent, CapturableContentFilter},
    capture_stream::{CaptureConfig, CaptureStream},
    feature::{bitmap::VideoFrameBitmap, screenshot},
};
use remoc::{codec, prelude::*};
use shadow_common::{
    client::{self as sc, PixelFormat, SystemPowerAction},
    error::ShadowError,
    server as ss,
};
use std::{path::Path, sync::Arc};
use sysinfo::System;
use tokio::{fs, io::AsyncReadExt, sync::RwLock};

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
        let mut list = fs::read_dir(&dir)
            .await
            .map_err(|err| ShadowError::QueryFilesError(dir.clone(), err.to_string()))?;

        while let Some(f) = list
            .next_entry()
            .await
            .map_err(|err| ShadowError::QueryFilesError(dir.clone(), err.to_string()))?
        {
            let name = match f.file_name().into_string() {
                Ok(n) => n,
                Err(_) => continue,
            };
            let file_type = match f.file_type().await {
                Ok(t) => t,
                Err(_) => continue,
            };
            let metadata = match f.metadata().await {
                Ok(m) => m,
                Err(_) => continue,
            };
            let size = metadata.len();

            ret.push(sc::File {
                name,
                is_dir: file_type.is_dir(),
                size,
            });
        }

        Ok(ret)
    }

    async fn get_displays(&self) -> Result<Vec<sc::Display>, ShadowError> {
        let content = CapturableContent::new(CapturableContentFilter::EVERYTHING).await?;
        Ok(content
            .displays()
            .map(|d| sc::Display { rect: d.rect() })
            .collect())
    }

    async fn get_pixel_formats(&self) -> Result<Vec<sc::PixelFormat>, ShadowError> {
        Ok(CaptureStream::supported_pixel_formats()
            .into_iter()
            .map(|p| PixelFormat { 0: *p })
            .collect())
    }

    async fn get_screenshot(
        &self,
        n_display: usize,
        format: sc::PixelFormat,
    ) -> Result<sc::Frame, ShadowError> {
        let token = match CaptureStream::test_access(false) {
            Some(t) => t,
            None => match CaptureStream::request_access(false).await {
                Some(t) => t,
                None => return Err(ShadowError::AccessDenied),
            },
        };

        let content = CapturableContent::new(CapturableContentFilter::EVERYTHING).await?;
        let display = match content.displays().nth(n_display) {
            Some(d) => d,
            None => return Err(ShadowError::NoSuchDisplay),
        };

        let config = CaptureConfig::with_display(display, format.0);
        let bitmap = screenshot::take_screenshot(token, config)
            .await?
            .get_bitmap()
            .map_err(|err| ShadowError::GetCapturableContentError(err.to_string()))?;

        Ok(match bitmap {
            crabgrab::feature::bitmap::FrameBitmap::BgraUnorm8x4(f) => sc::Frame {
                frame_type: sc::FrameType::BgraUnorm8x4(f.data.to_vec()),
                height: f.height,
                width: f.width,
            },
            crabgrab::feature::bitmap::FrameBitmap::RgbaUnormPacked1010102(f) => sc::Frame {
                frame_type: sc::FrameType::RgbaUnormPacked1010102(f.data.to_vec()),
                height: f.height,
                width: f.width,
            },
            crabgrab::feature::bitmap::FrameBitmap::RgbaF16x4(_) => {
                // TODO: Fix this

                return Err(ShadowError::Unsupported);
            }
            crabgrab::feature::bitmap::FrameBitmap::YCbCr(_) => {
                // TODO: Fix this

                return Err(ShadowError::Unsupported);
            }
        })
    }

    async fn get_file_content(&self, file_path: String) -> Result<Vec<u8>, ShadowError> {
        let mut buf = Vec::new();
        fs::File::open(file_path)
            .await?
            .read_to_end(&mut buf)
            .await?;

        Ok(buf)
    }

    async fn create_file(&self, file_path: String) -> Result<(), ShadowError> {
        fs::File::create(file_path).await?;

        Ok(())
    }
}
