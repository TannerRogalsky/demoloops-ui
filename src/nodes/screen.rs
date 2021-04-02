use crate::command::*;
use nodes::{FromAny, InputGroup, Node, NodeInput, NodeOutput, OneOrMany, PossibleInputs};
use std::any::{Any, TypeId};

struct ScreenInput {
    commands: Vec<Command>,
}

enum CommandInput {
    Command(nodes::OneOrMany<Command>),
    Draw(nodes::OneOrMany<DrawCommand>),
    Clear(nodes::OneOrMany<ClearCommand>),
}

fn command_types() -> [TypeId; 9] {
    let a = OneOrMany::<Command>::type_ids();
    let b = OneOrMany::<DrawCommand>::type_ids();
    let c = OneOrMany::<ClearCommand>::type_ids();
    [a[0], a[1], a[2], b[0], b[1], b[2], c[0], c[1], c[2]]
}

fn is_command(v: &dyn Any) -> bool {
    let ty = v.type_id();
    std::array::IntoIter::new(command_types()).any(|other| ty == other)
}

fn downcast(v: Box<dyn Any>) -> Result<CommandInput, Box<dyn Any>> {
    match OneOrMany::<Command>::downcast(v) {
        Ok(v) => Ok(CommandInput::Command(v)),
        Err(v) => match OneOrMany::<DrawCommand>::downcast(v) {
            Ok(v) => Ok(CommandInput::Draw(v)),
            Err(v) => OneOrMany::<ClearCommand>::downcast(v).map(CommandInput::Clear),
        },
    }
}

fn op(input: ScreenInput) -> Box<dyn Any> {
    Box::new(nodes::One::new(input.commands))
}

impl FromAny for ScreenInput {
    fn from_any(inputs: &mut Vec<Box<dyn Any>>) -> Result<Self, ()> {
        if inputs.iter().all(|i| is_command(&**i)) {
            let commands = inputs.drain(..).map(downcast).map(Result::unwrap);
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
}

impl nodes::InputSupplemental for ScreenInput {
    fn types(names: &'static [&str]) -> Vec<InputGroup<'static>> {
        std::array::IntoIter::new(command_types())
            .map(|type_id| InputGroup {
                info: vec![nodes::InputInfo {
                    name: names[0].into(),
                    ty_name: "Command",
                    type_id,
                }]
                .into(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ScreenNode;

impl NodeInput for ScreenNode {
    fn variadic(&self) -> bool {
        true
    }

    fn inputs(&self) -> PossibleInputs<'static> {
        use nodes::InputSupplemental;
        use once_cell::sync::Lazy;
        static GROUPS: Lazy<Vec<nodes::InputGroup<'static>>> =
            Lazy::new(|| ScreenInput::types(&["command"]));
        PossibleInputs::new(&*GROUPS)
    }
}

impl NodeOutput for ScreenNode {
    fn op(&self, inputs: &mut Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ()> {
        nodes::FromAny::from_any(inputs).map(op)
    }
}

#[typetag::serde]
impl Node for ScreenNode {
    fn name(&self) -> &'static str {
        "screen"
    }
}
