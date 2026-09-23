pub(crate) struct History {
    pub uri: String,
    pub back: Vec<String>,
    pub forward: Vec<String>,
}

impl Default for History {
    fn default() -> Self {
        Self {
            uri: String::from("about:blank"),
            back: vec![],
            forward: vec![],
        }
    }
}

impl History {
    pub fn previous(&mut self) -> Option<String> {
        if let Some(prev) = self.back.pop() {
            self.forward.push(self.uri.clone());
            self.uri = prev.clone();
            Some(prev)
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Option<String> {
        if let Some(next) = self.forward.pop() {
            self.back.push(self.uri.clone());
            self.uri = next.clone();
            Some(next)
        } else {
            None
        }
    }

    pub fn append(&mut self, uri: String) {
        self.back.push(self.uri.clone());
        self.uri = uri;
        self.forward = vec![];
    }

    pub fn has_previous(&self) -> bool {
        !self.back.is_empty()
    }

    pub fn has_next(&self) -> bool {
        !self.forward.is_empty()
    }
}
