use actix::prelude::*;

use crate::{
    audio_player::{Status, StatusRequest},
    tts_polly::Utterance,
    tts_polly::TtsPollyActor, interpreter::{Interpreter, Text},
};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Token(pub String);

pub struct TokenProcessorActor {
    token_buffer: Vec<String>,
    tts: Addr<TtsPollyActor>,
    idle: bool,

    state: ProcState,
    data_buffer: Vec<char>,

    interpreter: Addr<Interpreter>,
}

impl Actor for TokenProcessorActor {
    type Context = Context<Self>;
}

impl TokenProcessorActor {
    pub fn with(tts: Addr<TtsPollyActor>, interpreter: Addr<Interpreter>) -> Self {
        Self {
            token_buffer: vec![],
            tts,
            idle: true,
            state: ProcState::NotParsing,
            data_buffer: Vec::with_capacity(1024),
            interpreter,
        }
    }
}

impl Handler<Token> for TokenProcessorActor {
    type Result = ();

    fn handle(&mut self, msg: Token, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Token Proc   : Received token {}", msg.0);
        self.idle = false;

        let token = msg.0;

        // Find the next state and next thing to do
        let (next_state, next_action) = transition(self.state, self.data_buffer.as_slice(), &token);
        
        // Execute the next thing to do
        match next_action {
            Event::DoNothing => {}
            Event::Push { dst, pop } => {
                let data: String = self.data_buffer.drain(..(self.data_buffer.len() - pop)).collect();
                self.data_buffer.clear();
                match dst {
                    Dst::Speech => {
                        self.tts.do_send(Utterance(data))
                    },
                    Dst::Actions => {
                        self.interpreter.do_send(Text(data))
                    },
                }
            }
            Event::AddToBuffer => {
                self.data_buffer.extend(token.chars());
            }
        }
            
        // Update state
        self.state = next_state;
        self.idle = true;

        // println!("{:?}", self.state);
    }
}

impl Handler<StatusRequest> for TokenProcessorActor {
    type Result = Result<Status, std::io::Error>;

    fn handle(&mut self, _msg: StatusRequest, _ctx: &mut Context<Self>) -> Self::Result {
        // println!("Token Proc  : Received status request");

        if self.idle && self.token_buffer.is_empty() && self.data_buffer.is_empty() {
            Ok(Status::Idle)
        } else {
            Ok(Status::Busy)
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum ProcState {
    Parsing(ParseState),
    NotParsing,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum ParseState {
    Speech,
    Actions,
    Unknown { num_backticks: u8 },
}

enum Event {
    DoNothing,
    AddToBuffer,
    Push { dst: Dst, pop: usize },
}

enum Dst {
    Speech,
    Actions,
}

fn transition(
    current_state: ProcState,
    current_buffer: &[char],
    token: &str,
) -> (ProcState, Event) {
    use Event::*;
    use ParseState::*;
    use ProcState::*;

    // println!("Token: {}", token);

    match (current_state, current_buffer, token) {
        // Not parsing yet
        (NotParsing, _, "`") => (Parsing(Unknown { num_backticks: 1 }), DoNothing),
        (NotParsing, _, "``") => (Parsing(Unknown { num_backticks: 2 }), DoNothing),
        (NotParsing, _, "```") => (Parsing(Unknown { num_backticks: 3 }), DoNothing),
        (NotParsing, _, other) => {
            if other == "```speech" {
                (Parsing(Speech), DoNothing)
            } else if other == "```json" {
                (Parsing(Actions), DoNothing)
            } else if other.starts_with("```speech") {
                // TODO: Incorrect
                (Parsing(Speech), DoNothing)
            } else if other.starts_with("```json") {
                // TODO: Incorrect
                (Parsing(Actions), DoNothing)
            } else {
                panic!("Invalid state. Token: {}", token);
            }
        }

        // 1 backtick
        (Parsing(Unknown { num_backticks: 1 }), _, "``") => {
            (Parsing(Unknown { num_backticks: 3 }), DoNothing)
        }
        (Parsing(Unknown { num_backticks: 1 }), _, "``speech") => (Parsing(Speech), DoNothing),
        (Parsing(Unknown { num_backticks: 1 }), _, "``json") => (Parsing(Actions), DoNothing),

        // 2 backticks
        (Parsing(Unknown { num_backticks: 2 }), _, "`") => {
            (Parsing(Unknown { num_backticks: 3 }), DoNothing)
        }
        (Parsing(Unknown { num_backticks: 2 }), _, "`speech") => (Parsing(Speech), DoNothing),
        (Parsing(Unknown { num_backticks: 2 }), _, "`json") => (Parsing(Actions), DoNothing),

        // 3 backticks
        (Parsing(Unknown { num_backticks: 3 }), _, "speech") => (Parsing(Speech), DoNothing),
        (Parsing(Unknown { num_backticks: 3 }), _, "json") => (Parsing(Actions), DoNothing),

        // Other backticks
        (Parsing(Unknown { num_backticks }), _, token) => {
            panic!("Invalid {} {}", num_backticks, token)
        }

        // While parsing
        (Parsing(parse_state), _, "```") => match parse_state {
            Speech => (
                NotParsing,
                Push {
                    dst: Dst::Speech,
                    pop: 0,
                },
            ),
            Actions => (
                NotParsing,
                Push {
                    dst: Dst::Actions,
                    pop: 0,
                },
            ),
            Unknown { num_backticks: _ } => panic!(),
        },
        (Parsing(parse_state), current_buffer, "``" | "``\n" | "``\n\n") => {
            if current_buffer
                .last()
                .expect("current_buffer should not be empty")
                == &'`'
            {
                match parse_state {
                    Speech => (
                        NotParsing,
                        Push {
                            dst: Dst::Speech,
                            pop: 1,
                        },
                    ),
                    Actions => (
                        NotParsing,
                        Push {
                            dst: Dst::Actions,
                            pop: 1,
                        },
                    ),
                    Unknown { num_backticks: _ } => panic!(),
                }
            } else {
                (Parsing(parse_state), AddToBuffer)
            }
        }
        (Parsing(parse_state), current_buffer, "`" | "`\n" | "`\n\n") => {
            let last_two: String = current_buffer.iter().rev().take(2).collect();
            if last_two == "``" {
                match parse_state {
                    Speech => (
                        NotParsing,
                        Push {
                            dst: Dst::Speech,
                            pop: 2,
                        },
                    ),
                    Actions => (
                        NotParsing,
                        Push {
                            dst: Dst::Actions,
                            pop: 2,
                        },
                    ),
                    Unknown { num_backticks: _ } => panic!(),
                }
            } else {
                (Parsing(parse_state), AddToBuffer)
            }
        }
        (Parsing(parse_state), _, _) => (Parsing(parse_state), AddToBuffer),
    }
}
