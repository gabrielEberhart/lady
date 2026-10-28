use std::io;
use tokio::io::{AsyncRead, AsyncWrite};

/// Anything that can carry bytes both ways.
pub trait IpcStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> IpcStream for T {}

/// One endpoint abstraction: A calls `bind`/`accept`, B calls `connect`.
#[allow(async_fn_in_trait)]
pub trait IpcEndpoint: Sized {
    type Server: IpcStream; // stream returned to the listener side
    type Client: IpcStream; // stream returned to the connecting side

    fn bind(name: &str) -> io::Result<Self>;
    async fn accept(&mut self) -> io::Result<Self::Server>;
    async fn connect(name: &str) -> io::Result<Self::Client>;
}

// ---------- Unix ----------
#[cfg(unix)]
mod platform {
    use super::*;
    use std::path::PathBuf;
    use tokio::net::{UnixListener, UnixStream};

    fn path_for(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{name}.sock"))
    }

    pub struct SocketEndpoint {
        listener: UnixListener,
        path: PathBuf,
    }

    impl IpcEndpoint for SocketEndpoint {
        type Server = UnixStream;
        type Client = UnixStream;

        fn bind(name: &str) -> io::Result<Self> {
            let path = path_for(name);
            let _ = std::fs::remove_file(&path); // clear stale socket
            Ok(Self {
                listener: UnixListener::bind(&path)?,
                path,
            })
        }

        async fn accept(&mut self) -> io::Result<UnixStream> {
            Ok(self.listener.accept().await?.0)
        }

        async fn connect(name: &str) -> io::Result<UnixStream> {
            UnixStream::connect(path_for(name)).await
        }
    }

    impl Drop for SocketEndpoint {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

// ---------- Windows ----------
#[cfg(windows)]
mod platform {
    use super::*;
    use std::time::Duration;
    use tokio::net::windows::named_pipe::{
        ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
    };

    const ERROR_PIPE_BUSY: i32 = 231;

    fn pipe_name(name: &str) -> String {
        return format!(r"\\.\pipe\{name}");
    }

    pub struct PipeEndpoint {
        name: String,
        next: NamedPipeServer, // pre-created instance waiting for a client
    }

    impl IpcEndpoint for PipeEndpoint {
        type Server = NamedPipeServer;
        type Client = NamedPipeClient;

        fn bind(name: &str) -> io::Result<Self> {
            let name = pipe_name(name);
            let next = ServerOptions::new()
                .first_pipe_instance(true)
                .create(&name)?;
            Ok(Self { name, next })
        }

        async fn accept(&mut self) -> io::Result<NamedPipeServer> {
            self.next.connect().await?;
            // Create the next instance before handing this one out,
            // so new clients never see "pipe not found".
            let fresh = ServerOptions::new().create(&self.name)?;
            Ok(std::mem::replace(&mut self.next, fresh))
        }

        async fn connect(name: &str) -> io::Result<NamedPipeClient> {
            let name = pipe_name(name);
            loop {
                match ClientOptions::new().open(&name) {
                    Ok(c) => return Ok(c),
                    Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                        tokio::time::sleep(Duration::from_millis(50)).await
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }
}

#[cfg(windows)]
pub use platform::PipeEndpoint as PlatformEndpoint;
#[cfg(unix)]
pub use platform::SocketEndpoint as PlatformEndpoint;
