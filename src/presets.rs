fn init<'a, 'b, const CLASS: char>() -> (
    std::io::StdinLock<'static>,
    std::io::StdoutLock<'static>,
    crate::Term<'a, 'b, CLASS>,
    String,
) {
    (
        std::io::stdin().lock(),
        std::io::stdout().lock(),
        crate::Term::new(),
        String::new(),
    )
}
