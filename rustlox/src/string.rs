use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Handle(usize);

impl Handle {
  pub fn as_str(&self) -> InternedString {
    with_interner(|interner| unsafe {
      InternedString {
        string: ::std::mem::transmute::<&str, &str>(interner.get(self.0)),
      }
    })
  }

  pub fn from_str(string: &str) -> Handle {
    with_interner(|interner| interner.intern(string))
  }
}

impl ops::Add<&Handle> for &Handle {
  type Output = Handle;
  fn add(self, other: &Handle) -> <Self as std::ops::Add<&Handle>>::Output {
    let string = format!("{}{}", self, other);
    Handle::from_str(string.as_str())
  }
}

impl Display for Handle {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    write!(f, "{}", self.as_str().string)
  }
}

#[derive(Default)]
struct Interner {
  handle_map: HashMap<Box<str>, Handle>,
  strings: Vec<Box<str>>,
}

impl Interner {
  fn new() -> Interner {
    Interner::default()
  }

  fn intern(&mut self, string: &str) -> Handle {
    if let Some(&handle) = self.handle_map.get(string) {
      return handle;
    }

    let handle = Handle(self.strings.len());
    let string = string.to_string().into_boxed_str();
    self.strings.push(string.clone());
    self.handle_map.insert(string, handle);
    handle
  }

  fn get(&self, index: usize) -> &str {
    &self.strings[index]
  }
}

fn with_interner<T, F: FnOnce(&mut Interner) -> T>(f: F) -> T {
  thread_local!(static INTERNER: RefCell<Interner> = {
      RefCell::new(Interner::new())
  });
  INTERNER.with(|interner| f(&mut *interner.borrow_mut()))
}

pub struct InternedString {
  pub string: &'static str,
}
