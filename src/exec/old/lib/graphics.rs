// struct DoubleBufferState {
//     active: Option<Vec<u32>>,
//     next: RwLock<Option<Vec<u32>>>,
// }

// pub struct DoubleBuffer {
//     state: RwLock<DoubleBufferState>,
// }

// impl DoubleBuffer {
//     pub fn new(size: usize) -> Self {
//         Self {
//             state: RwLock::new(DoubleBufferState {
//                 active: Some(vec![0; size]),
//                 next: RwLock::new(Some(vec![0; size])),
//             }),
//         }
//     }

//     pub fn flip(&self) {
//         let mut state = &mut *self.state.write().unwrap();
//         let mut next = state.next.write().unwrap();

//         let active = state.active.take();
//         state.active = next.take();
//         *next = active;
//     }

//     fn read_active<T, F: FnOnce(&[u32]) -> T>(&self, f: F) -> T {
//         let state = &*self.state.read().unwrap();
//         f(state.active.as_ref().unwrap())
//     }

//     fn write_next<F: FnOnce(&mut [u32])>(&self, f: F) {
//         let state = &mut *self.state.write().unwrap();
//         let mut next = state.next.write().unwrap();
//         f(next.as_mut().unwrap())
//     }
// }
