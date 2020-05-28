use super::types::{Backend, Commander, Poller, PollerError, PollerFactory};
use mpsc::{RecvError, SendError, TryRecvError};
use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver, Sender},
};

impl<Command> Commander<Command> for Sender<Command> {
    fn send(&self, cmd: Command) -> Result<(), SendError<Command>> {
        self.send(cmd)
    }
}

pub struct Blocking<B: Backend> {
    backend: B,
    cmds: Receiver<B::Command>,
    rsps: VecDeque<Result<B::Response, B::Error>>,
}

pub struct BlockingFactory;

impl<B: Backend> PollerFactory<B> for BlockingFactory {
    type Commander = Sender<B::Command>;
    type Poller = Blocking<B>;

    fn poller<F: FnOnce() -> Result<B, B::Error>>(
        &self,
        backend_new: F,
    ) -> Result<(Self::Commander, Self::Poller), B::Error> {
        let (cmd_in, cmd_out) = mpsc::channel();

        Ok((
            cmd_in,
            Blocking {
                backend: backend_new()?,
                cmds: cmd_out,
                rsps: VecDeque::new(),
            },
        ))
    }
}

impl<B: Backend> Blocking<B> {
    fn pump_one(&mut self) {
        // RUSTFIX does this drop the failures?
        let mut cmds: Vec<B::Command> = self.cmds.try_iter().collect();
        if cmds.is_empty() {
            cmds = self.cmds.recv().into_iter().collect();
        }

        for cmd in cmds {
            if let Some(rsp) = self.backend.process(cmd).transpose() {
                self.rsps.push_back(rsp);
            }
        }
    }
}

impl<B: Backend> Poller<B> for Blocking<B> {
    fn recv(&mut self) -> Result<B::Response, PollerError<B::Error>> {
        if self.rsps.is_empty() {
            self.pump_one();
        }

        match self.rsps.pop_front() {
            None => Err(PollerError::Shutdown),
            Some(Err(error)) => Err(PollerError::Backend(error)),
            Some(Ok(rsp)) => Ok(rsp),
        }
    }

    fn try_recv(&mut self) -> Result<Option<B::Response>, PollerError<B::Error>> {
        self.recv().map(Some)
    }
}

pub trait TaskSpawner {
    fn spawn<F: FnOnce() + Send + 'static>(&self, f: F);
}

pub struct ThreadSpawner;

impl TaskSpawner for ThreadSpawner {
    fn spawn<F: FnOnce() + Send + 'static>(&self, f: F) {
        std::thread::spawn(f);
    }
}

pub struct Async<B: Backend> {
    rsps: Receiver<Result<B::Response, B::Error>>,
}

pub struct AsyncFactory<S: TaskSpawner> {
    spawner: S,
}

impl<S: TaskSpawner> AsyncFactory<S> {
    pub fn new(spawner: S) -> Self {
        Self { spawner }
    }

    fn pump_loop<B: Backend>(
        backend: Result<B, B::Error>,
        cmds: Receiver<B::Command>,
        rsps: Sender<Result<B::Response, B::Error>>,
    ) {
        let mut backend = match backend {
            Ok(backend) => backend,
            Err(error) => {
                rsps.send(Err(error)).ok();
                return;
            }
        };

        loop {
            match cmds.recv() {
                Err(RecvError) => break,
                Ok(cmd) => {
                    if let Some(rsp) = backend.process(cmd).transpose() {
                        if rsps.send(rsp).is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }
}

impl<B: Backend + 'static, S: TaskSpawner> PollerFactory<B> for AsyncFactory<S> {
    type Commander = Sender<B::Command>;
    type Poller = Async<B>;

    fn poller<F: FnOnce() -> Result<B, B::Error> + Send + 'static>(
        &self,
        backend_new: F,
    ) -> Result<(Self::Commander, Self::Poller), B::Error> {
        let (cmd_in, cmd_out) = mpsc::channel();
        let (rsp_in, rsp_out) = mpsc::channel();

        self.spawner
            .spawn(|| Self::pump_loop(backend_new(), cmd_out, rsp_in));

        Ok((cmd_in, Async { rsps: rsp_out }))
    }
}

impl<B: Backend> Poller<B> for Async<B> {
    fn recv(&mut self) -> Result<B::Response, PollerError<B::Error>> {
        match self.rsps.recv() {
            Err(RecvError) => Err(PollerError::Shutdown),
            Ok(Err(error)) => Err(PollerError::Backend(error)),
            Ok(Ok(rsp)) => Ok(rsp),
        }
    }

    fn try_recv(&mut self) -> Result<Option<B::Response>, PollerError<B::Error>> {
        match self.rsps.try_recv() {
            Err(TryRecvError::Disconnected) => Err(PollerError::Shutdown),
            Err(TryRecvError::Empty) => Ok(None),
            Ok(Err(error)) => Err(PollerError::Backend(error)),
            Ok(Ok(rsp)) => Ok(Some(rsp)),
        }
    }
}
