fn init<'a, 'b>() -> (
    std::io::StdinLock<'static>,
    std::io::StdoutLock<'static>,
    crate::Term<'a, 'b>,
    String,
) {
    (
        std::io::stdin().lock(),
        std::io::stdout().lock(),
        crate::Term::new(),
        String::new(),
    )
}
