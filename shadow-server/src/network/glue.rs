use crate::network::ServerObj;
use rch::oneshot::Receiver;
use remoc::{codec::Bincode, prelude::*};
use shadow_common::{
    client::{self as sc, Client},
    CallResult,
};
use std::net::SocketAddr;

impl ServerObj {
    #[inline]
    pub async fn system_power(&self, action: sc::SystemPowerAction) -> CallResult<()> {
        self.get_client().await?.system_power(action).await
    }

    #[inline]
    pub async fn get_installed_apps(&self) -> CallResult<Vec<sc::App>> {
        self.get_client().await?.get_installed_apps().await
    }

    #[inline]
    pub async fn get_processes(&self) -> CallResult<Vec<sc::Process>> {
        self.get_client().await?.get_processes().await
    }

    #[inline]
    pub async fn get_file_list<S: AsRef<str>>(&self, dir: S) -> CallResult<Vec<sc::File>> {
        self.get_client()
            .await?
            .get_file_list(dir.as_ref().into())
            .await
    }

    #[inline]
    pub async fn get_file_content<S: AsRef<str>>(&self, file: S) -> CallResult<Vec<u8>> {
        self.get_client()
            .await?
            .get_file_content(file.as_ref().into())
            .await
    }

    #[inline]
    pub async fn create_file<S: AsRef<str>>(&self, file: S) -> CallResult<()> {
        self.get_client()
            .await?
            .create_file(file.as_ref().into())
            .await
    }

    #[inline]
    pub async fn open_file<S: AsRef<str>>(&self, file: S) -> CallResult<sc::Execute> {
        self.get_client()
            .await?
            .open_file(file.as_ref().into())
            .await
    }

    #[inline]
    pub async fn create_dir<S: AsRef<str>>(&self, dir: S) -> CallResult<()> {
        self.get_client()
            .await?
            .create_dir(dir.as_ref().into())
            .await
    }

    #[inline]
    pub async fn write_file<S: AsRef<str>>(&self, file: S, content: Vec<u8>) -> CallResult<()> {
        self.get_client()
            .await?
            .write_file(file.as_ref().into(), content)
            .await
    }

    #[inline]
    pub async fn delete_file<S: AsRef<str>>(&self, file: S) -> CallResult<()> {
        self.get_client()
            .await?
            .delete_file(file.as_ref().into())
            .await
    }

    #[inline]
    pub async fn delete_dir_recursive<S: AsRef<str>>(&self, dir: S) -> CallResult<()> {
        self.get_client()
            .await?
            .delete_dir_recursive(dir.as_ref().into())
            .await
    }

    #[inline]
    pub async fn kill_process(&self, pid: u32) -> CallResult<()> {
        self.get_client().await?.kill_process(pid).await
    }

    #[inline]
    pub async fn get_display_info(&self) -> CallResult<Vec<sc::Display>> {
        self.get_client().await?.get_display_info().await
    }

    #[inline]
    pub async fn proxy(
        &self,
        target_addr: SocketAddr,
        sender: rch::bin::Sender,
        receiver: rch::bin::Receiver,
    ) -> CallResult<Receiver<bool, Bincode>> {
        self.get_client()
            .await?
            .proxy(target_addr, sender, receiver)
            .await
    }
}
