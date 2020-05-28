mod renderer;
mod window;

use crate::{
    exec::types::{
        Backend, EventLoop as TraitEventLoop, Frontend, FrontendError, Poller, PollerError, Vram,
    },
    vm,
};
use itertools::iproduct;
use renderer::bitmap;
use std::marker::PhantomData;
use window::{Ticker as TraitTicker, Window, WindowContext};

const PIXEL_WIDTH: usize = 8;

struct Ticker<Monitor, B, F, P> {
    _marker: PhantomData<B>,

    poller: P,
    frontend: F,
    buff: Vec<u8>,
    last_monitor: Option<Monitor>,
}

impl<Monitor, B, F, P> Ticker<Monitor, B, F, P> {
    fn new(poller: P, frontend: F) -> Self {
        Self {
            _marker: PhantomData,

            poller,
            frontend,
            buff: vec![0; 4 * vm::VIDEO_WIDTH * vm::VIDEO_HEIGHT * PIXEL_WIDTH * PIXEL_WIDTH],
            last_monitor: None,
        }
    }

    // /// `buff` is 4 bytes, `vram` is `2` words.
    // #[inline]
    // fn set_pixel(buff: &mut [u8], vram: &[u16]) {
    //     buff[0] = (vram[0] & 0x00FF) as u8;
    //     buff[1] = ((vram[0] & 0xFF00) >> 8) as u8;
    //     buff[2] = (vram[1] & 0x00FF) as u8;
    //     buff[3] = ((vram[1] & 0xFF00) >> 8) as u8;
    // }

    // /// `buff` is the entire video buffer, `vram` is `2` words.
    // #[inline]
    // fn set_pixel_block(x: usize, y: usize, buff: &mut [u8], vram: &[u16]) {
    //     for (i, j) in iproduct!(0..PIXEL_WIDTH, 0..PIXEL_WIDTH) {
    //         let point = (y * PIXEL_WIDTH + j) * PIXEL_WIDTH * vm::VIDEO_WIDTH + (x * PIXEL_WIDTH + i);
    //         Self::set_pixel(&mut buff[point*4..(point + 1)*4], vram);
    //     }
    // }

    // fn update_buff(&mut self, vram: Vram) {
    //     for (x, y) in iproduct!(0..vm::VIDEO_WIDTH, 0..vm::VIDEO_HEIGHT) {
    //         let point = y * vm::VIDEO_WIDTH + x;
    //         Self::set_pixel_block(x, y, &mut self.buff, &vram.0[point*2..(point + 1)*2]);
    //     }
    // }

    /// `buff` is 4 bytes, `vram` is `2` words.
    #[inline]
    fn set_pixel(buff_off: usize, vram_off: usize, buff: &mut [u8], vram: &[u16]) {
        buff[buff_off + 0] = (vram[vram_off + 0] & 0x00FF) as u8;
        buff[buff_off + 1] = ((vram[vram_off + 0] & 0xFF00) >> 8) as u8;
        buff[buff_off + 2] = (vram[vram_off + 1] & 0x00FF) as u8;
        buff[buff_off + 3] = ((vram[vram_off + 1] & 0xFF00) >> 8) as u8;
    }

    /// `buff` is the entire video buffer, `vram` is `2` words.
    #[inline]
    fn set_pixel_block(x: usize, y: usize, vram_off: usize, buff: &mut [u8], vram: &[u16]) {
        for (i, j) in iproduct!(0..PIXEL_WIDTH, 0..PIXEL_WIDTH) {
            let point =
                (y * PIXEL_WIDTH + j) * PIXEL_WIDTH * vm::VIDEO_WIDTH + (x * PIXEL_WIDTH + i);
            Self::set_pixel(point * 4, vram_off, buff, vram);
        }
    }

    fn update_buff(&mut self, vram: Vram) {
        for (x, y) in iproduct!(0..vm::VIDEO_WIDTH, 0..vm::VIDEO_HEIGHT) {
            let point = y * vm::VIDEO_WIDTH + x;
            Self::set_pixel_block(x, y, point * 2, &mut self.buff, &vram.0);
        }
    }
}

impl<Monitor, B: Backend, F: Frontend<(Monitor, Vram), Response = B::Response>, P: Poller<B>>
    TraitTicker<Monitor, PollerError<B::Error>> for Ticker<Monitor, B, F, P>
{
    type Builder = bitmap::Builder;

    fn builder(&self) -> bitmap::Builder {
        bitmap::Builder::new(
            (vm::VIDEO_WIDTH * PIXEL_WIDTH) as u32,
            (vm::VIDEO_HEIGHT * PIXEL_WIDTH) as u32,
        )
    }

    fn tick(
        &mut self,
        renderer: &mut bitmap::Renderer,
        ctx: &WindowContext,
    ) -> Result<(), PollerError<B::Error>> {
        if let Some(rsp) = self.poller.try_recv()? {
            match self.frontend.process(rsp) {
                Err(FrontendError::Shutdown) => Err(PollerError::Shutdown)?,
                Err(FrontendError::Nothing) => (),
                Ok((monitor, vram)) => {
                    self.last_monitor = Some(monitor);
                    self.update_buff(vram);

                    renderer.set_bitmap(ctx, &self.buff);
                }
            }
        }

        Ok(())
    }

    fn report(
        self,
        terminal: Option<PollerError<B::Error>>,
    ) -> Result<Monitor, PollerError<B::Error>> {
        match terminal {
            None | Some(PollerError::Shutdown) => {
                self.frontend.teardown();
                self.last_monitor.ok_or(PollerError::Shutdown)
            }
            Some(error) => Err(error),
        }
    }
}

pub struct EventLoop {
    window: Window,
}

impl EventLoop {
    pub fn new() -> Self {
        Self {
            window: Window::new("Video Display"),
        }
    }
}

impl<Monitor> TraitEventLoop<Monitor> for EventLoop {
    type Monitor = (Monitor, Vram);

    fn run<B: Backend, F: Frontend<(Monitor, Vram), Response = B::Response>, P: Poller<B>>(
        self,
        poller: P,
        frontend: F,
    ) -> Result<Monitor, PollerError<B::Error>> {
        self.window.run(Ticker::new(poller, frontend))
    }
}
