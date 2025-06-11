trait MyFunc {
    type Fixture;
    fn myfunction(&self, fixture: &mut Self::Fixture);
}

struct FunctionHolder<T> {
    func: fn(&mut T),
}

impl<T> MyFunc for FunctionHolder<T> {
    type Fixture = T;
    fn myfunction(&self, fixture: &mut Self::Fixture) {
        (self.func)(fixture)
    }
}

fn func1(fixture: &mut i32) {
    println!("{:?}", fixture);
}

fn func2(fixture: &mut i32) {
    println!("{:?}", fixture);
}

fn main() {
    let funcs: Vec<Box<FunctionHolder<i32>>> = vec![
        Box::new(FunctionHolder { func: func1 }),
        Box::new(FunctionHolder { func: func2 }),
    ];

    let mut value = 0;
    for func in funcs {
        func.myfunction(&mut value);
    }

    // context.bar
}
