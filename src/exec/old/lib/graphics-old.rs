use minifb::{Key, Window, WindowOptions};
use std::{
    sync::{Arc, Mutex, RwLock},
    thread, time,
};
use thread::JoinHandle;

struct DoubleBufferState {
    active: Option<Vec<u32>>,
    next: RwLock<Option<Vec<u32>>>,
}

pub struct DoubleBuffer {
    state: RwLock<DoubleBufferState>,
}

impl DoubleBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            state: RwLock::new(DoubleBufferState {
                active: Some(vec![0; size]),
                next: RwLock::new(Some(vec![0; size])),
            }),
        }
    }

    pub fn flip(&self) {
        let mut state = &mut *self.state.write().unwrap();
        let mut next = state.next.write().unwrap();

        let active = state.active.take();
        state.active = next.take();
        *next = active;
    }

    fn read_active<T, F: FnOnce(&[u32]) -> T>(&self, f: F) -> T {
        let state = &*self.state.read().unwrap();
        f(state.active.as_ref().unwrap())
    }

    fn write_next<F: FnOnce(&mut [u32])>(&self, f: F) {
        let state = &mut *self.state.write().unwrap();
        let mut next = state.next.write().unwrap();
        f(next.as_mut().unwrap())
    }
}

#[derive(Clone)]
struct FrameDescriptor {
    width: usize,
    height: usize,
    buff: Arc<Mutex<Option<DoubleBuffer>>>,
}

impl FrameDescriptor {
    fn window_loop(self, name: String) {
        let mut window = Window::new(&name, self.width, self.height, WindowOptions::default())
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });
        drop(name);

        // We don't want to be blocking with the buffer mutex held.
        window.limit_update_rate(Some(time::Duration::from_nanos(0)));
        while window.is_open() && !window.is_key_down(Key::Escape) {
            let then = time::Instant::now();

            match self.buff.lock().unwrap().as_ref() {
                None => break,
                Some(buff) => buff.read_active(|active| {
                    window
                        .update_with_buffer(active, self.width, self.height)
                        .unwrap()
                }),
            }

            let left = time::Duration::from_micros(1666).checked_sub(then.elapsed());
            if let Some(left) = left {
                thread::sleep(left);
            }
        }
    }
}

pub struct Frame {
    thread: Option<JoinHandle<()>>,
    handle: Arc<Mutex<Option<DoubleBuffer>>>,
}

impl Frame {
    pub fn new(name: &str, width: usize, height: usize, headless: bool) -> Self {
        let buff = Arc::new(Mutex::new(Some(DoubleBuffer::new(width * height))));

        let thread = if headless {
            None
        } else {
            let frame = FrameDescriptor {
                width,
                height,
                buff: buff.clone(),
            };
            let name_owned = name.to_owned();
            Some(thread::spawn(|| frame.window_loop(name_owned)))
        };

        Self { thread, handle: buff }
    }

    pub fn flip(&self) {
        self.handle.lock().unwrap().as_ref().unwrap().flip();
    }

    pub fn read_active<T, F: FnOnce(&[u32]) -> T>(&self, f: F) -> T {
        self.handle.lock().unwrap().as_ref().unwrap().read_active(f)
    }

    pub fn write_next<F: FnOnce(&mut [u32])>(&self, f: F) {
        self.handle.lock().unwrap().as_ref().unwrap().write_next(f);
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        let mut lock = self.handle.lock().unwrap();
        *lock = None;
        drop(lock);
        // self.thread.take().map(|thread| thread.join());
        // thread::sleep_ms(10);
    }
}