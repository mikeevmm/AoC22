use super::{Program, Command};

/// A struct to dispatch commands from an inner `Program`, but simulating blocking from command
/// processing.
pub struct BlockingDispatcher<P: Program> {
    blocking: u8,
    inner: P,
    next: Command,
    pub done: bool,
}

pub enum BlockingState {
    /// This tick will block, and the `BlockingDispatcher` should be polled again at the start of
    /// the next tick.
    Blocked,
    /// This command finished running at the end of the last tick and should be dispatched
    /// immediately.
    Completed(Command),
}

#[derive(Debug)]
pub enum DispatcherError {
    EmptyProgram,
}

impl<P: Program> BlockingDispatcher<P> {
    pub fn from_program(program: P) -> Result<Self, DispatcherError> {
        let mut dispatcher = BlockingDispatcher {
            blocking: 0, // Dummy value.
            inner: program,
            next: Command::NoOp, // Dummy value.
            done: false,
        };
        let first_cmd = dispatcher
            .inner
            .next()
            .ok_or(DispatcherError::EmptyProgram)?;
        dispatcher.load_cmd(first_cmd);
        Ok(dispatcher)
    }

    /// Polls the CPU command at the **start** of each tick.
    ///
    /// For example, the following program
    ///     
    ///     noop
    ///     noop
    ///     addx 1
    ///
    /// will yield the following `poll` call results:
    ///
    ///     BlockingState::Completed(Command::NoOp) // Second tick: noop has finished
    ///                                             // and next noop has begun dispatching
    ///     BlockingState::Completed(Command::NoOp) // Third tick: noop has finished and
    ///                                             // and addx has begun dispatching
    ///     BlockingState::Blocked  // Third tick: addx is blocking
    ///     BlockingState::Completed(Command::AddX(1)) // Fourth tick: addx completed
    ///     
    /// Note how the first call to `tick` immediately places us at the start of the *second* tick.
    /// I.e., in the following schematic
    ///
    ///         [ noop ] [ noop ] [ noop ] [ addx ]
    ///                  ^
    ///                  
    /// Calling `tick` returns the state at the time indicated by `^`.
    pub fn tick(&mut self) -> Option<BlockingState> {
        let result = if self.blocking > 0 {
            self.blocking -= 1;
            Some(BlockingState::Blocked)
        } else {
            // Prepare the return value...
            let next_cmd = self.inner.next();
            if next_cmd.is_none() {
                self.done = true;
            }
            let done = self.load_cmd(next_cmd.unwrap_or(Command::NoOp)); // Will set `next` and `blocking`
            Some(BlockingState::Completed(done))
        };
        result
    }

    fn load_cmd(&mut self, cmd: Command) -> Command {
        self.blocking = match cmd {
            Command::AddX(_) => 1, // Will block next tick
            Command::NoOp => 0,    // Will be dispatched next tick
        };
        let replaced = std::mem::replace(&mut self.next, cmd);
        replaced
    }
}
