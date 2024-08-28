// if feature 'presets' is on then the crate only compiles the basic presets,
// in this case, cache is only a vec of History values that get saved to a file,
// otherwise cache is a libsql db that can house many caches, not just input Histories

use crate::object_tree::ObjectTree;

fn init() -> (
    std::io::StdinLock<'static>,
    std::io::StdoutLock<'static>,
    ObjectTree,
    String,
) {
    (
        std::io::stdin().lock(),
        std::io::stdout().lock(),
        ObjectTree::new(),
        String::new(),
    )
}
