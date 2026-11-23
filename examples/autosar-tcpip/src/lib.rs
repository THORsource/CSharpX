#![forbid(unsafe_code)]

pub mod frame_device;

use std::collections::BTreeMap;

use smoltcp::{
    iface::{Config, Interface, SocketHandle, SocketSet},
    phy::{Device, Medium},
    socket::{tcp, udp},
    time::{Duration, Instant},
    wire::{EthernetAddress, IpCidr, IpEndpoint},
};

const MAX_SOCKETS: usize = 8;
const BUFFER_BYTES: usize = 4096;
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Offline,
    Startup,
    Online,
    OnHold,
    Shutdown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddressState {
    Assigned,
    OnHold,
    Unassigned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Indication {
    Mode(Mode),
    Address(AddressState),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SocketId(u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidConfiguration,
    InvalidMode,
    InvalidSocket,
    SocketLimit,
    SocketRejected,
    WouldBlock,
    EndOfStream,
}

pub struct TcpIp<D> {
    eth_if: D,
    interface: Interface,
    address: IpCidr,
    mode: Mode,
    requested_mode: Mode,
    shutdown_deadline: Instant,
    sockets: SocketSet<'static>,
    handles: BTreeMap<SocketId, (Protocol, SocketHandle)>,
    next_id: u32,
}

impl<D: Device> TcpIp<D> {
    pub fn init(
        mut eth_if: D,
        mac: EthernetAddress,
        address: IpCidr,
        random_seed: u64,
        now: Instant,
    ) -> Result<Self, Error> {
        if eth_if.capabilities().medium != Medium::Ethernet
            || !mac.is_unicast()
            || !address.address().is_unicast()
        {
            return Err(Error::InvalidConfiguration);
        }
        let mut config = Config::new(mac.into());
        config.random_seed = random_seed;
        let interface = Interface::new(config, &mut eth_if, now);
        Ok(Self {
            eth_if,
            interface,
            address,
            mode: Mode::Offline,
            requested_mode: Mode::Offline,
            shutdown_deadline: now,
            sockets: SocketSet::new(Vec::new()),
            handles: BTreeMap::new(),
            next_id: 0,
        })
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn ethernet(&mut self) -> &mut D {
        &mut self.eth_if
    }

    pub fn request_com_mode(&mut self, mode: Mode) -> Result<(), Error> {
        match mode {
            Mode::Online | Mode::Offline => {}
            Mode::OnHold if self.mode == Mode::Online => {}
            _ => return Err(Error::InvalidMode),
        }
        self.requested_mode = mode;
        Ok(())
    }

    pub fn main_function(&mut self, now: Instant) -> Vec<Indication> {
        let mut indications = Vec::new();
        let previous = self.mode;
        match (self.mode, self.requested_mode) {
            (Mode::Offline, Mode::Online) => self.mode = Mode::Startup,
            (Mode::Startup, Mode::Online) => {
                self.interface.update_ip_addrs(|addrs| {
                    addrs.clear();
                    addrs
                        .push(self.address)
                        .expect("interface has room for one address");
                });
                self.mode = Mode::Online;
                indications.push(Indication::Address(AddressState::Assigned));
            }
            (Mode::Startup, Mode::Offline) => self.mode = Mode::Offline,
            (Mode::Online, Mode::OnHold) => {
                self.mode = Mode::OnHold;
                indications.push(Indication::Address(AddressState::OnHold));
            }
            (Mode::OnHold, Mode::Online) => {
                self.mode = Mode::Online;
                indications.push(Indication::Address(AddressState::Assigned));
            }
            (Mode::Online | Mode::OnHold, Mode::Offline) => {
                indications.push(Indication::Address(AddressState::Unassigned));
                let immediate = self.mode == Mode::OnHold;
                self.begin_shutdown(now, immediate);
            }
            (Mode::Shutdown, _) => {
                let has_open_tcp = self.handles.values().any(|(protocol, handle)| {
                    *protocol == Protocol::Tcp && self.sockets.get::<tcp::Socket>(*handle).is_open()
                });
                if !has_open_tcp || now >= self.shutdown_deadline {
                    for (_, handle) in std::mem::take(&mut self.handles).into_values() {
                        self.sockets.remove(handle);
                    }
                    self.interface.update_ip_addrs(|addrs| addrs.clear());
                    self.mode = Mode::Offline;
                }
            }
            _ => {}
        }
        if matches!(self.mode, Mode::Online | Mode::Shutdown) {
            self.interface
                .poll(now, &mut self.eth_if, &mut self.sockets);
        }
        if previous != self.mode {
            indications.push(Indication::Mode(self.mode));
        }
        indications
    }

    fn begin_shutdown(&mut self, now: Instant, immediate: bool) {
        self.shutdown_deadline = now + SHUTDOWN_TIMEOUT;
        self.mode = Mode::Shutdown;
        self.handles.retain(|_, (protocol, handle)| {
            if *protocol == Protocol::Udp || immediate {
                self.sockets.remove(*handle);
                return false;
            }
            self.sockets.get_mut::<tcp::Socket>(*handle).close();
            true
        });
    }

    fn require_online(&self) -> Result<(), Error> {
        if self.mode == Mode::Online {
            Ok(())
        } else {
            Err(Error::InvalidMode)
        }
    }

    fn socket_handle(&self, id: SocketId, protocol: Protocol) -> Result<SocketHandle, Error> {
        self.require_online()?;
        match self.handles.get(&id) {
            Some((actual, handle)) if *actual == protocol => Ok(*handle),
            _ => Err(Error::InvalidSocket),
        }
    }

    pub fn get_socket(&mut self, protocol: Protocol) -> Result<SocketId, Error> {
        self.require_online()?;
        if self.handles.len() == MAX_SOCKETS {
            return Err(Error::SocketLimit);
        }
        let id = SocketId(self.next_id);
        self.next_id = self.next_id.checked_add(1).ok_or(Error::SocketLimit)?;
        let handle = match protocol {
            Protocol::Tcp => self.sockets.add(tcp::Socket::new(
                tcp::SocketBuffer::new(vec![0; BUFFER_BYTES]),
                tcp::SocketBuffer::new(vec![0; BUFFER_BYTES]),
            )),
            Protocol::Udp => self.sockets.add(udp::Socket::new(
                udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY; 4], vec![0; BUFFER_BYTES]),
                udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY; 4], vec![0; BUFFER_BYTES]),
            )),
        };
        self.handles.insert(id, (protocol, handle));
        Ok(id)
    }

    pub fn udp_bind(&mut self, id: SocketId, port: u16) -> Result<(), Error> {
        let handle = self.socket_handle(id, Protocol::Udp)?;
        self.sockets
            .get_mut::<udp::Socket>(handle)
            .bind(port)
            .map_err(|_| Error::SocketRejected)
    }

    pub fn tcp_listen(&mut self, id: SocketId, port: u16) -> Result<(), Error> {
        let handle = self.socket_handle(id, Protocol::Tcp)?;
        self.sockets
            .get_mut::<tcp::Socket>(handle)
            .listen(port)
            .map_err(|_| Error::SocketRejected)
    }

    pub fn tcp_connect(
        &mut self,
        id: SocketId,
        remote: IpEndpoint,
        port: u16,
    ) -> Result<(), Error> {
        let handle = self.socket_handle(id, Protocol::Tcp)?;
        self.sockets
            .get_mut::<tcp::Socket>(handle)
            .connect(self.interface.context(), remote, port)
            .map_err(|_| Error::SocketRejected)
    }

    pub fn tcp_is_connected(&self, id: SocketId) -> Result<bool, Error> {
        let handle = self.socket_handle(id, Protocol::Tcp)?;
        Ok(self.sockets.get::<tcp::Socket>(handle).state() == tcp::State::Established)
    }

    pub fn tcp_transmit(&mut self, id: SocketId, data: &[u8]) -> Result<usize, Error> {
        let handle = self.socket_handle(id, Protocol::Tcp)?;
        let socket = self.sockets.get_mut::<tcp::Socket>(handle);
        if !socket.may_send() {
            return Err(Error::SocketRejected);
        }
        if !data.is_empty() && !socket.can_send() {
            return Err(Error::WouldBlock);
        }
        socket.send_slice(data).map_err(|_| Error::SocketRejected)
    }

    pub fn tcp_receive(&mut self, id: SocketId, data: &mut [u8]) -> Result<usize, Error> {
        let handle = self.socket_handle(id, Protocol::Tcp)?;
        let socket = self.sockets.get_mut::<tcp::Socket>(handle);
        if !socket.can_recv() {
            return if socket.may_recv() {
                Err(Error::WouldBlock)
            } else {
                Err(Error::EndOfStream)
            };
        }
        socket.recv_slice(data).map_err(|_| Error::SocketRejected)
    }

    pub fn udp_transmit(
        &mut self,
        id: SocketId,
        remote: IpEndpoint,
        data: &[u8],
    ) -> Result<(), Error> {
        let handle = self.socket_handle(id, Protocol::Udp)?;
        // The example uses Ethernet MTU 1500 without IPv4 fragmentation.
        if data.len() > 1472 || remote.port == 0 {
            return Err(Error::SocketRejected);
        }
        self.sockets
            .get_mut::<udp::Socket>(handle)
            .send_slice(data, remote)
            .map_err(|error| match error {
                udp::SendError::BufferFull => Error::WouldBlock,
                udp::SendError::Unaddressable => Error::SocketRejected,
            })
    }

    pub fn udp_receive(
        &mut self,
        id: SocketId,
        data: &mut [u8],
    ) -> Result<(usize, IpEndpoint), Error> {
        let handle = self.socket_handle(id, Protocol::Udp)?;
        self.sockets
            .get_mut::<udp::Socket>(handle)
            .recv_slice(data)
            .map(|(length, metadata)| (length, metadata.endpoint))
            .map_err(|error| match error {
                udp::RecvError::Exhausted => Error::WouldBlock,
                udp::RecvError::Truncated => Error::SocketRejected,
            })
    }

    pub fn abort_socket(&mut self, id: SocketId) -> Result<(), Error> {
        self.require_online()?;
        let (_, handle) = self.handles.remove(&id).ok_or(Error::InvalidSocket)?;
        self.sockets.remove(handle);
        Ok(())
    }
}
