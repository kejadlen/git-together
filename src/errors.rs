use std::process::Output;

error_chain! {
  errors {
    GitConfig(output: Output) {
      description("git config error")
      display("git config error ({:?}): '{}'",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).trim(),
      )
    }
    AuthorNotFound(init: String) {
      description("author not found")
      display("author not found: '{}'", init)
    }
    InvalidAuthor(raw: String) {
      description("invalid author")
      display("invalid author: '{}'", raw)
    }
  }
}
