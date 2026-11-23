#![forbid(unsafe_code)]

use csharpx_autosar_tcpip::{
    frame_device::FrameDevice, AddressState, Error, Indication, Mode, Protocol, TcpIp,
};
use smoltcp::{
    time::Instant,
    wire::{
        EthernetAddress, EthernetFrame, EthernetProtocol, IpAddress, IpCidr, IpProtocol,
        Ipv4Packet, TcpPacket,
    },
};

type Node = TcpIp<FrameDevice>;

#[derive(Default)]
struct Wire {
    arp: usize,
    syn: usize,
    syn_ack: usize,
    fin: usize,
    udp: usize,
}

impl Wire {
    fn forward(&mut self, from: &mut Node, to: &mut Node) {
        while let Some(data) = from.ethernet().take_frame() {
            let frame = EthernetFrame::new_checked(&data).unwrap();
            match frame.ethertype() {
                EthernetProtocol::Arp => self.arp += 1,
                EthernetProtocol::Ipv4 => {
                    let packet = Ipv4Packet::new_checked(frame.payload()).unwrap();
                    match packet.next_header() {
                        IpProtocol::Tcp => {
                            let tcp = TcpPacket::new_checked(packet.payload()).unwrap();
                            if tcp.syn() && tcp.ack() {
                                self.syn_ack += 1;
                            } else if tcp.syn() {
                                self.syn += 1;
                            }
                            if tcp.fin() {
                                self.fin += 1;
                            }
                        }
                        IpProtocol::Udp => self.udp += 1,
                        _ => {}
                    }
                }
                _ => {}
            }
            to.ethernet()
                .receive_frame(data)
                .expect("virtual link queue full");
        }
    }

    fn cycle(&mut self, left: &mut Node, right: &mut Node, tick: i64) {
        let now = Instant::from_millis(tick);
        left.main_function(now);
        self.forward(left, right);
        right.main_function(now);
        self.forward(right, left);
    }
}

fn node(host: u8) -> Node {
    TcpIp::init(
        FrameDevice::default(),
        EthernetAddress([2, 0, 0, 0, 0, host]),
        IpCidr::new(IpAddress::v4(192, 0, 2, host), 24),
        u64::from(host),
        Instant::ZERO,
    )
    .unwrap()
}

fn bring_online(node: &mut Node) {
    assert_eq!(node.get_socket(Protocol::Tcp), Err(Error::InvalidMode));
    assert_eq!(
        node.request_com_mode(Mode::Startup),
        Err(Error::InvalidMode)
    );
    node.request_com_mode(Mode::Online).unwrap();
    assert_eq!(
        node.main_function(Instant::ZERO),
        [Indication::Mode(Mode::Startup)]
    );
    assert_eq!(
        node.main_function(Instant::from_millis(1)),
        [
            Indication::Address(AddressState::Assigned),
            Indication::Mode(Mode::Online)
        ]
    );
}

fn scenario() -> Wire {
    let mut client = node(1);
    let mut server = node(2);
    bring_online(&mut client);
    bring_online(&mut server);

    let remote = IpAddress::v4(192, 0, 2, 2);
    let listener = server.get_socket(Protocol::Tcp).unwrap();
    server.tcp_listen(listener, 7000).unwrap();
    let connection = client.get_socket(Protocol::Tcp).unwrap();
    client
        .tcp_connect(connection, (remote, 7000).into(), 50000)
        .unwrap();

    let mut wire = Wire::default();
    for tick in 2..25 {
        wire.cycle(&mut client, &mut server, tick);
    }
    assert!(client.tcp_is_connected(connection).unwrap());
    assert!(server.tcp_is_connected(listener).unwrap());
    assert!(wire.arp >= 2 && wire.syn > 0 && wire.syn_ack > 0);

    let data = b"vehicle-speed=42";
    assert_eq!(client.tcp_transmit(connection, data).unwrap(), data.len());
    for tick in 25..40 {
        wire.cycle(&mut client, &mut server, tick);
    }
    let mut received = [0; 128];
    let length = server.tcp_receive(listener, &mut received).unwrap();
    assert_eq!(&received[..length], data);

    let udp_in = server.get_socket(Protocol::Udp).unwrap();
    server.udp_bind(udp_in, 7001).unwrap();
    let udp_out = client.get_socket(Protocol::Udp).unwrap();
    client.udp_bind(udp_out, 50001).unwrap();

    client.request_com_mode(Mode::OnHold).unwrap();
    assert_eq!(
        client.main_function(Instant::from_millis(40)),
        [
            Indication::Address(AddressState::OnHold),
            Indication::Mode(Mode::OnHold)
        ]
    );
    assert_eq!(
        client.tcp_transmit(connection, b"blocked"),
        Err(Error::InvalidMode)
    );
    assert_eq!(
        client.udp_transmit(udp_out, (remote, 7001).into(), b"blocked"),
        Err(Error::InvalidMode)
    );
    assert!(client.ethernet().take_frame().is_none());
    client.request_com_mode(Mode::Online).unwrap();
    assert_eq!(
        client.main_function(Instant::from_millis(41)),
        [
            Indication::Address(AddressState::Assigned),
            Indication::Mode(Mode::Online)
        ]
    );
    assert!(client.tcp_is_connected(connection).unwrap());
    let resumed = b"vehicle-speed=43";
    assert_eq!(
        client.tcp_transmit(connection, resumed).unwrap(),
        resumed.len()
    );
    client
        .udp_transmit(udp_out, (remote, 7001).into(), b"heartbeat=1")
        .unwrap();
    for tick in 42..60 {
        wire.cycle(&mut client, &mut server, tick);
    }

    let length = server.tcp_receive(listener, &mut received).unwrap();
    assert_eq!(&received[..length], resumed);
    let (length, sender) = server.udp_receive(udp_in, &mut received).unwrap();
    assert_eq!(&received[..length], b"heartbeat=1");
    assert_eq!(sender, (IpAddress::v4(192, 0, 2, 1), 50001).into());
    assert!(wire.udp > 0);

    client.request_com_mode(Mode::Offline).unwrap();
    server.request_com_mode(Mode::Offline).unwrap();
    wire.cycle(&mut client, &mut server, 60);
    assert_eq!(client.mode(), Mode::Shutdown);
    assert_eq!(server.mode(), Mode::Shutdown);
    for tick in (61..12_000).step_by(10) {
        wire.cycle(&mut client, &mut server, tick);
        if client.mode() == Mode::Offline && server.mode() == Mode::Offline {
            break;
        }
    }
    assert_eq!(client.mode(), Mode::Offline);
    assert_eq!(server.mode(), Mode::Offline);
    assert!(wire.fin >= 2);

    client.request_com_mode(Mode::Online).unwrap();
    client.main_function(Instant::from_millis(12_001));
    client.main_function(Instant::from_millis(12_002));
    assert_eq!(
        client.tcp_is_connected(connection),
        Err(Error::InvalidSocket)
    );
    wire
}

fn main() {
    let wire = scenario();
    println!(
        "Verified Ethernet wire: {} ARP, {} TCP SYN, {} TCP SYN/ACK, {} TCP FIN, {} UDP",
        wire.arp, wire.syn, wire.syn_ack, wire.fin, wire.udp
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ethernet_tcp_udp_and_controller_lifecycle() {
        scenario();
    }

    #[test]
    fn offline_from_on_hold_discards_sockets_without_transmitting() {
        let mut controller = node(1);
        bring_online(&mut controller);
        let socket = controller.get_socket(Protocol::Tcp).unwrap();
        controller.tcp_listen(socket, 7000).unwrap();
        controller.request_com_mode(Mode::OnHold).unwrap();
        controller.main_function(Instant::from_millis(2));
        controller.request_com_mode(Mode::Offline).unwrap();
        controller.main_function(Instant::from_millis(3));
        controller.main_function(Instant::from_millis(4));
        assert_eq!(controller.mode(), Mode::Offline);
        assert!(controller.ethernet().take_frame().is_none());
    }
}
