enum test {
    A(i32),
    B(bool),
}

fn test_fun(x: test) -> bool {
    match x {
        test::A(n) => n < 5,
        test::B(b) => b,
    }
}