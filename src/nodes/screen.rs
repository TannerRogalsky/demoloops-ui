use crate::command::*;
use nodes::{FromAnyProto, InputComponent, InputGroup, InputStack, OneOrMany, PossibleInputs};
use std::any::Any;

struct ScreenInput {
    commands: Vec<Command>,
}

#[derive(InputComponent)]
enum CommandInput {
    Command(nodes::OneOrMany<Command>),
    Draw(nodes::OneOrMany<DrawCommand>),
    Clear(nodes::OneOrMany<ClearCommand>),
}

fn op(input: ScreenInput) -> Box<dyn Any> {
    Box::new(nodes::One::new(input.commands))
}

impl FromAnyProto for ScreenInput {
    fn from_any(inputs: InputStack<'_, Box<dyn Any>>) -> Result<Self, ()> {
        if inputs.deref_iter().all(CommandInput::is) {
            let commands = inputs
                .consume()
                .map(CommandInput::downcast)
                .map(Result::unwrap);
            // TODO: capacity might be calculable by summing all size_hints
            let mut acc = Vec::new();
            for command in commands {
                match command {
                    CommandInput::Command(command) => match command {
                        OneOrMany::One(v) => acc.push(v.inner()),
                        OneOrMany::Many(v) => acc.extend(v.inner()),
                    },
                    CommandInput::Draw(command) => match command {
                        OneOrMany::One(v) => acc.push(Command::Draw(v.inner())),
                        OneOrMany::Many(v) => acc.extend(v.inner().map(Command::Draw)),
                    },
                    CommandInput::Clear(command) => match command {
                        OneOrMany::One(v) => acc.push(Command::Clear(v.inner())),
                        OneOrMany::Many(v) => acc.extend(v.inner().map(Command::Clear)),
                    },
                }
            }
            Ok(Self { commands: acc })
        } else {
            Err(())
        }
    }

    fn possible_inputs(names: &'static [&str]) -> PossibleInputs<'static> {
        let groups = CommandInput::type_ids()
            .into_iter()
            .map(|type_id| InputGroup {
                info: vec![nodes::InputInfo {
                    name: names[0].into(),
                    ty_name: "Command",
                    type_id,
                    optional: false,
                }]
                .into(),
            })
            .collect::<Vec<_>>();
        PossibleInputs::new(groups)
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ScreenNode;

impl nodes::NodeInput for ScreenNode {
    fn variadic(&self) -> bool {
        true
    }

    fn inputs(&self) -> PossibleInputs<'static> {
        use once_cell::sync::Lazy;
        static CACHE: Lazy<PossibleInputs> =
            Lazy::new(|| ScreenInput::possible_inputs(&["command"]));
        PossibleInputs::new(&*CACHE.groups)
    }
}

impl nodes::NodeOutput for ScreenNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        nodes::FromAnyProto::from_any(nodes::InputStack::new(inputs, ..)).map(op)
    }
}

#[typetag::serde]
impl nodes::Node for ScreenNode {
    fn name(&self) -> &'static str {
        "screen"
    }
}
