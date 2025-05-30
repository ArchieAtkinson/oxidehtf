struct Foo<'a> {
    vec: Vec<i32>,
    current: Option<&'a i32>,
}

fn main() {
    let mut foo = Foo {
        vec: vec![1, 2, 3, 4],
        current: None,
    };

    foo.current = Some(&foo.vec[0]);

    println!("{:?}", foo.current);

    foo.vec[0] = 5;

    println!("{:?}", foo.current);
}
