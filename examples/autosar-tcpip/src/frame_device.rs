use std::collections::VecDeque;

use smoltcp::{
    phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken},
    time::Instant,
};

const FRAME_LIMIT: usize = 32;
const FRAME_SIZE: usize = 1514;

#[derive(Default)]
pub struct FrameDevice {
    rx: VecDeque<Vec<u8>>,
    tx: VecDeque<Vec<u8>>,
}

impl FrameDevice {
    pub fn receive_frame(&mut self, frame: Vec<u8>) -> Result<(), Vec<u8>> {
        if self.rx.len() >= FRAME_LIMIT || !(14..=FRAME_SIZE).contains(&frame.len()) {
            return Err(frame);
        }
        self.rx.push_back(frame);
        Ok(())
    }

    pub fn take_frame(&mut self) -> Option<Vec<u8>> {
        self.tx.pop_front()
    }
}

pub struct Incoming(Vec<u8>);

impl RxToken for Incoming {
    fn consume<R, F: FnOnce(&[u8]) -> R>(self, f: F) -> R {
        f(&self.0)
    }
}

pub struct Outgoing<'a>(&'a mut VecDeque<Vec<u8>>);

impl TxToken for Outgoing<'_> {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(self, len: usize, f: F) -> R {
        let mut frame = vec![0; len];
        let result = f(&mut frame);
        self.0.push_back(frame);
        result
    }
}

impl Device for FrameDevice {
    type RxToken<'a> = Incoming;
    type TxToken<'a> = Outgoing<'a>;

    fn receive(&mut self, _: Instant) -> Option<(Incoming, Outgoing<'_>)> {
        if self.tx.len() >= FRAME_LIMIT {
            return None;
        }
        Some((Incoming(self.rx.pop_front()?), Outgoing(&mut self.tx)))
    }

    fn transmit(&mut self, _: Instant) -> Option<Outgoing<'_>> {
        (self.tx.len() < FRAME_LIMIT).then_some(Outgoing(&mut self.tx))
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = FRAME_SIZE;
        caps.max_burst_size = Some(FRAME_LIMIT);
        caps
    }
}
