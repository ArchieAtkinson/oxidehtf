use std::ops::Deref;

struct User {
    bar: i32,
}

struct Context<T> {
    foo: i32,
    user: T,
}

impl<T> Deref for Context<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

fn main() {
    let context = Context {
        foo: 5,
        user: User { bar: 6 },
    };

    context.bar
}
