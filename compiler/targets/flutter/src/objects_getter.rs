use alloc::{
  collections::BTreeSet,
  fmt::{self, Write},
  string::String,
  vec::Vec,
};
use dropin_compiler_recipes::ir::{Component, Format, FormatInner, KeyFormat};

use crate::{Stage, Stated};

#[derive(Debug)]
pub struct ObjectGetter<'a, S>
where
  S: Stage,
{
  sub: &'a S,
  state: ObjectGetterState<'a>,
}

impl<'a, S> ObjectGetter<'a, S>
where
  S: Stage + 'a,
{
  pub fn new(sub: &'a S) -> Self {
    let state = ObjectGetterState::new(sub);
    Self { sub, state }
  }
}

impl<'a, S> Stage for ObjectGetter<'a, S>
where
  S: Stage,
{
  fn ir(&self) -> &Component {
    self.sub.ir()
  }
}

impl<'a, S> Stated<ObjectGetterState<'a>> for ObjectGetter<'a, S>
where
  S: Stage,
{
  fn state(&self) -> &ObjectGetterState<'a> {
    &self.state
  }
}

#[derive(Debug)]
pub struct ObjectGetterState<'a> {
  pub objects: BTreeSet<Vec<&'a str>>,
}

impl<'a> ObjectGetterState<'a> {
  pub fn new<S>(sub: &'a S) -> Self
  where
    S: Stage,
  {
    let mut objects = BTreeSet::new();

    let ir = sub.ir();
    let mut iters = Vec::new();
    iters.push(FormatStackNode::Keys(
      ir.variables.as_ref().unwrap().keys.iter(),
    ));
    let mut keys = Vec::new();

    while !iters.is_empty() {
      let node = iters.last_mut().unwrap();
      let (key, format): (&str, &Format) = match node {
        FormatStackNode::Keys(iter) => {
          let Some(key) = iter.next() else {
            iters.pop();
            keys.pop();
            continue;
          };
          let format = key.format.as_ref().unwrap();
          (&key.key, format)
        }
        FormatStackNode::Format(format) => {
          let Some(format) = format.take() else {
            iters.pop();
            continue;
          };
          ("*", format)
        }
      };
      let format = format.format_inner.as_ref().unwrap();
      match format {
        FormatInner::Index(sub) => {
          iters
            .push(FormatStackNode::Format(Some(sub.format.as_ref().unwrap())));
          keys.push(key);
        }
        FormatInner::List(sub) => {
          iters
            .push(FormatStackNode::Format(Some(sub.format.as_ref().unwrap())));
          keys.push(key);
        }
        FormatInner::Object(sub) => {
          iters.push(FormatStackNode::Keys(sub.keys.iter()));
          keys.push(key);
          objects.insert(keys.clone());
        }
        _ => {}
      }
    }

    Self { objects }
  }
}

enum FormatStackNode<'a, I>
where
  I: Iterator<Item = &'a KeyFormat>,
{
  Keys(I),
  Format(Option<&'a Format>),
}

pub fn write_class_name(output: &mut String, trace: &[&str]) -> fmt::Result {
  for key in trace {
    match *key {
      "*" => {
        write!(output, "_")?;
      }
      "_" => write!(output, "__")?,
      _ => {
        let mut is_capital = true;
        for c in key.chars() {
          if c == '_' {
            is_capital = true;
            continue;
          }
          if is_capital {
            output.push(c.to_ascii_uppercase());
          } else {
            output.push(c);
          }
          is_capital = false;
        }
      }
    }
  }
  Ok(())
}
