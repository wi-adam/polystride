use crate::model::{MAX_FRAME, Request, Response};
use crate::store::{CoordinatorConfig, Store};
use crate::{Result, error};
use rustls::pki_types::ServerName;
use rustls::{
    ClientConfig, ClientConnection, RootCertStore, ServerConfig, ServerConnection, StreamOwned,
};
use serde::{Serialize, de::DeserializeOwned};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// Serialize a bounded JSON frame with its four-byte length prefix.
///
/// # Errors
///
/// Returns an error on serialization, length-limit, or stream-write failure.
pub fn write_frame<T: Serialize>(stream: &mut impl Write, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > MAX_FRAME {
        return Err(error("protocol frame exceeds size limit"));
    }
    stream.write_all(&u32::try_from(bytes.len())?.to_be_bytes())?;
    stream.write_all(&bytes)?;
    stream.flush()?;
    Ok(())
}

/// Read and decode one bounded length-prefixed JSON frame.
///
/// # Errors
///
/// Returns an error for incomplete/oversized frames, invalid JSON, or read failures.
pub fn read_frame<T: DeserializeOwned>(stream: &mut impl Read) -> Result<T> {
    let mut header = [0; 4];
    stream.read_exact(&mut header)?;
    let size = u32::from_be_bytes(header) as usize;
    if size > MAX_FRAME {
        return Err(error("protocol frame exceeds size limit"));
    }
    let mut bytes = vec![0; size];
    stream.read_exact(&mut bytes)?;
    Ok(serde_json::from_slice(&bytes)?)
}

/// Configure server authentication using the explicitly supplied CA certificates.
///
/// # Errors
///
/// Returns an error if the CA file is unreadable, empty, or contains invalid certificates.
pub fn client_config(ca: &Path) -> Result<Arc<ClientConfig>> {
    let mut roots = RootCertStore::empty();
    for certificate in rustls_pemfile::certs(&mut BufReader::new(File::open(ca)?)) {
        roots.add(certificate?)?;
    }
    if roots.is_empty() {
        return Err(error("CA file contains no certificates"));
    }
    Ok(Arc::new(
        ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    ))
}

/// Load the coordinator's TLS certificate chain and private key.
///
/// # Errors
///
/// Returns an error if files cannot be read or the certificate/key configuration is invalid.
pub fn server_config(config: &CoordinatorConfig) -> Result<Arc<ServerConfig>> {
    let certificates = rustls_pemfile::certs(&mut BufReader::new(File::open(&config.certificate)?))
        .collect::<std::io::Result<Vec<_>>>()?;
    let key = rustls_pemfile::private_key(&mut BufReader::new(File::open(&config.private_key)?))?
        .ok_or_else(|| error("no private key in PEM file"))?;
    Ok(Arc::new(
        ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certificates, key)?,
    ))
}

fn timeouts(socket: &TcpStream) -> Result<()> {
    socket.set_read_timeout(Some(Duration::from_secs(15)))?;
    socket.set_write_timeout(Some(Duration::from_secs(15)))?;
    Ok(())
}

/// Send one request over an authenticated outbound TLS connection.
///
/// # Errors
///
/// Returns an error for DNS, connection, socket, TLS, or framing failures.
pub fn exchange(
    address: &str,
    server_name: &str,
    tls: Arc<ClientConfig>,
    request: &Request,
) -> Result<Response> {
    let addresses = address.to_socket_addrs()?;
    let mut connected = None;
    for address in addresses {
        if let Ok(socket) = TcpStream::connect_timeout(&address, Duration::from_secs(10)) {
            connected = Some(socket);
            break;
        }
    }
    let socket = connected.ok_or_else(|| error("could not connect to coordinator"))?;
    timeouts(&socket)?;
    let connection = ClientConnection::new(tls, ServerName::try_from(server_name.to_owned())?)?;
    let mut stream = StreamOwned::new(connection, socket);
    write_frame(&mut stream, request)?;
    read_frame(&mut stream)
}

/// Handle one TLS request and write its response.
///
/// # Errors
///
/// Returns an error for socket/TLS/framing failures or failed store operations.
pub fn serve_connection(socket: TcpStream, tls: Arc<ServerConfig>, store: &Store) -> Result<()> {
    timeouts(&socket)?;
    let connection = ServerConnection::new(tls)?;
    let mut stream = StreamOwned::new(connection, socket);
    let request: Request = read_frame(&mut stream)?;
    let response = store.handle(request)?;
    write_frame(&mut stream, &response)
}

/// Run the sequential coordinator using an exclusive state-directory lock.
///
/// # Errors
///
/// Returns an error if the store, state lock, TLS configuration, or listener
/// cannot be initialized. Per-connection failures are logged and do not stop it.
pub fn serve(config: &CoordinatorConfig) -> Result<()> {
    let store = Store::open(&config.state_dir, config.hosts.clone())?;
    let _lock = crate::agent::lock_directory(&config.state_dir)?;
    let tls = server_config(config)?;
    let listener = TcpListener::bind(&config.listen)?;
    eprintln!("coordinator listening on {}", listener.local_addr()?);
    // A deliberately small, sequential research coordinator. Socket operations
    // have idle timeouts; this is not a public Internet service.
    for socket in listener.incoming() {
        match socket {
            Ok(socket) => {
                if serve_connection(socket, tls.clone(), &store).is_err() {
                    // Do not log frames, tokens, or submitted environment/output.
                    eprintln!("connection failed; request was not acknowledged");
                }
            }
            Err(_) => eprintln!("connection could not be accepted"),
        }
    }
    Ok(())
}
