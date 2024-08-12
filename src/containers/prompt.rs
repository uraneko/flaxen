#[derive(Debug, Default)]
pub struct Prompt {
    values: Vec<char>,
}

impl Prompt {
    fn new(values: Option<&str>) -> Self {
        Self {
            values: values
                .unwrap_or("")
                .chars()
                .into_iter()
                .collect::<Vec<char>>(),
        }
    }

    fn overwrite(&mut self, new_val: &str) {
        self.values.clear();
        new_val
            .chars()
            .into_iter()
            .for_each(|c| self.values.push(c));
    }

    fn clear(&mut self) {
        self.values.clear();
    }
}
