mod dispatcher;
pub mod crt;

type RegisterValue = isize;

pub struct Register {
    inner: isize,
}

pub struct Clock {
    inner: usize,
}

#[derive(Debug)]
pub enum Command {
    AddX(RegisterValue),
    NoOp,
}

pub trait Program: Iterator<Item = Command> {}
impl<T> Program for T where T: Iterator<Item = Command> {}

pub struct Cpu<P: Program> {
    register: Register,
    clock: Clock,
    dispatcher: dispatcher::BlockingDispatcher<P>,
}

impl Register {
    fn starting() -> Self {
        Register { inner: 1 }
    }

    fn add(&mut self, rhs: isize) {
        self.inner += rhs;
    }

    pub fn value(&self) -> isize {
        self.inner
    }
}

impl Clock {
    fn starting() -> Self {
        Clock { inner: 0 }
    }

    fn tick(&mut self) {
        self.inner += 1;
    }

    pub fn count_finished(&self) -> usize {
        self.inner
    }

    pub fn current_frame(&self) -> usize {
        self.inner + 1
    }
}

impl<P: Program> Cpu<P> {
    pub fn load(program: P) -> Self {
        Cpu {
            register: Register::starting(),
            clock: Clock::starting(),
            dispatcher: dispatcher::BlockingDispatcher::from_program(program)
                .expect("non-empty program"),
        }
    }
    
    /// Runs the CPU until the **start** of the next tick.
    pub fn finish_frame(&mut self) {
        self.clock.tick();
        if let Some(dispatcher::BlockingState::Completed(cmd)) = self.dispatcher.tick() {
            self.run_cmd(cmd);
        }
    }

    fn run_cmd(&mut self, cmd: Command) {
        match cmd {
            Command::AddX(x) => self.register.add(x),
            Command::NoOp => {}
        }
    }

    pub fn x_register(&self) -> &Register {
        &self.register
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    pub fn done(&self) -> bool {
        self.dispatcher.done
    }
}
