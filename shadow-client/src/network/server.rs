use crate::misc;
use display_info::DisplayInfo;
use local_encoding::{Encoder, Encoding};
use log::debug;
use rch::oneshot::{self, Receiver};
use remoc::{
    codec::{self, Bincode},
    prelude::*,
};
use shadow_common::{client as sc, error::ShadowError, misc::sender, server as ss, CallResult};
use shlex::Shlex;
use std::{net::SocketAddr, path::Path, sync::Arc};
use sysinfo::{Pid, System};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    process::Command,
    sync::RwLock,
};

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
    async fn handshake(&self) -> CallResult<sc::Handshake> {
        Ok(sc::Handshake {
            message: format!("{:#?}", self.cfg),
        })
    }

    async fn get_system_info(&self) -> CallResult<sc::SystemInfo> {
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

    async fn system_power(&self, action: sc::SystemPowerAction) -> CallResult<()> {
        match action {
            sc::SystemPowerAction::Shutdown => system_shutdown::shutdown(),
            sc::SystemPowerAction::Reboot => system_shutdown::reboot(),
            sc::SystemPowerAction::Logout => system_shutdown::logout(),
            sc::SystemPowerAction::Sleep => system_shutdown::sleep(),
            sc::SystemPowerAction::Hibernate => system_shutdown::hibernate(),
        }
        .map_err(|_| ShadowError::SystemPowerError)?;

        Ok(())
    }

    async fn get_installed_apps(&self) -> CallResult<Vec<sc::App>> {
        Ok(installed::list()
            .map_err(|e| ShadowError::QueryAppsError(e.to_string()))?
            .filter(|app| app.name().to_string().is_empty() == false)
            .map(|app| sc::App {
                name: app.name().to_string(),
                publisher: app.publisher().to_string(),
                version: app.version().to_string(),
            })
            .collect())
    }

    async fn get_processes(&self) -> CallResult<Vec<sc::Process>> {
        let mut system = System::new();
        system.refresh_processes();

        Ok(system
            .processes()
            .iter()
            .map(|(pid, process)| {
                let parent_pid = process.parent().map(|v| v.as_u32());

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

    async fn get_file_list(&self, dir: String) -> CallResult<Vec<sc::File>> {
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

    async fn get_file_content(&self, file_path: String) -> CallResult<Vec<u8>> {
        let mut buf = Vec::new();
        fs::File::open(file_path)
            .await?
            .read_to_end(&mut buf)
            .await?;

        Ok(buf)
    }

    async fn create_file(&self, file_path: String) -> CallResult<()> {
        fs::File::create(file_path).await?;

        Ok(())
    }

    async fn write_file(&self, file_path: String, content: Vec<u8>) -> CallResult<()> {
        let mut file = fs::OpenOptions::new().write(true).open(file_path).await?;
        file.write_all(&content).await?;
        file.flush().await?;

        Ok(())
    }

    async fn delete_file(&self, file_path: String) -> CallResult<()> {
        fs::remove_file(file_path).await?;

        Ok(())
    }

    async fn delete_dir_recursive(&self, dir_path: String) -> CallResult<()> {
        fs::remove_dir_all(dir_path).await?;

        Ok(())
    }

    async fn create_dir(&self, dir_path: String) -> CallResult<()> {
        fs::create_dir_all(dir_path).await?;

        Ok(())
    }

    async fn kill_process(&self, pid: u32) -> CallResult<()> {
        match System::new_all()
            .process(Pid::from_u32(pid))
            .ok_or(ShadowError::ProcessNotFound(pid.to_string()))?
            .kill()
        {
            true => Ok(()),
            false => Err(ShadowError::UnknownError),
        }
    }

    async fn open_file(&self, file_path: String) -> CallResult<sc::Execute> {
        let mut lex = Shlex::new(&file_path);
        let app = lex
            .next()
            .ok_or(ShadowError::ParamInvalid("no file specified".into()))?;
        let mut command = Command::new(app);

        lex.for_each(|arg| {
            command.arg(arg);
        });

        let result = command.output().await?;
        let status = result.status.to_string();
        let output = Encoding::OEM.to_string(&result.stdout)?;

        Ok(sc::Execute { status, output })
    }

    async fn get_display_info(&self) -> CallResult<Vec<sc::Display>> {
        Ok(DisplayInfo::all()?
            .into_iter()
            .map(|i| sc::Display::from(i))
            .collect())
    }

    async fn proxy(
        &self,
        target_addr: SocketAddr,
        sender: rch::bin::Sender,
        receiver: rch::bin::Receiver,
    ) -> CallResult<Receiver<bool, Bincode>> {
        let stream = TcpStream::connect(target_addr).await?;
        let (signal_tx, signal_rx) = oneshot::channel();

        tokio::spawn(async move {
            sender::transfer(
                sender.into_inner().await?,
                receiver.into_inner().await?,
                stream,
            )
            .await;

            signal_tx.send(true)?;
            debug!("outbound {} proxy exited", target_addr);

            Ok::<(), anyhow::Error>(())
        });

        Ok(signal_rx)
    }
}
