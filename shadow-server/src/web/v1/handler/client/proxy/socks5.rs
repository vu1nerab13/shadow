use shadow_common::{error::ShadowError, CallResult};
use socks5_impl::{
    protocol::{
        handshake, Address, AsyncStreamOperation, AuthMethod, Command, Reply, Request, Response,
    },
    server::{auth, AuthExecutor},
};
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::net::TcpStream;

pub async fn parse(
    stream: &mut TcpStream,
    user: String,
    password: String,
) -> CallResult<SocketAddr> {
    let request = handshake::Request::retrieve_from_async_stream(stream).await?;
    if request.evaluate_method(AuthMethod::UserPass) {
        handshake::Response::new(AuthMethod::UserPass)
            .write_to_async_stream(stream)
            .await?;

        if auth::UserKeyAuth::new(&user, &password)
            .execute(stream)
            .await?
            == false
        {
            return Err(ShadowError::AccessDenied);
        };
    } else {
        handshake::Response::new(AuthMethod::NoAcceptableMethods)
            .write_to_async_stream(stream)
            .await?;

        return Err(ShadowError::ParamInvalid(
            "no available handshake method provided by client".into(),
        ));
    };

    let request = Request::retrieve_from_async_stream(stream).await?;

    let ret = match request.command {
        Command::Bind | Command::UdpAssociate => {
            Response::new(Reply::CommandNotSupported, Address::unspecified())
                .write_to_async_stream(stream)
                .await?;

            Err(ShadowError::Unsupported)
        }
        Command::Connect => match request.address {
            Address::DomainAddress(domain, port) => Ok((domain.clone(), port)
                .to_socket_addrs()?
                .next()
                .ok_or(ShadowError::DnsLookupError(domain))?),
            Address::SocketAddress(addr) => Ok(addr),
        },
    };

    Response::new(Reply::Succeeded, Address::unspecified())
        .write_to_async_stream(stream)
        .await?;

    ret
}
