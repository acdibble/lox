use crate::chunk::Chunk;
use crate::string;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Copy, Clone, Debug)]
pub struct Handle(usize);

pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: string::Handle,
}

impl Handle {
    pub fn get(&self) -> &mut Function {
        with_interner(|interner| unsafe {
            ::std::mem::transmute::<&mut Function, &mut Function>(interner.get(self.0))
        })
    }

    pub fn from(func: Function) -> Handle {
        with_interner(|interner| interner.intern(func))
    }
}

impl Display for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.get().name)
    }
}

#[derive(Default)]
struct Interner {
    handle_map: HashMap<Box<str>, Handle>,
    functions: Vec<Box<Function>>,
}

impl Interner {
    fn new() -> Interner {
        Interner::default()
    }

    fn intern(&mut self, function: Function) -> Handle {
        let name = function.name.to_string().into_boxed_str();
        if let Some(&handle) = self.handle_map.get(&name) {
            return handle;
        }

        let handle = Handle(self.functions.len());
        self.functions.push(Box::from(function));
        self.handle_map.insert(name, handle);
        handle
    }

    fn get(&mut self, index: usize) -> &mut Function {
        &mut self.functions[index]
    }
}

fn with_interner<T, F: FnOnce(&mut Interner) -> T>(f: F) -> T {
    thread_local!(static INTERNER: RefCell<Interner> = {
        RefCell::new(Interner::new())
    });
    INTERNER.with(|interner| f(&mut *interner.borrow_mut()))
}
