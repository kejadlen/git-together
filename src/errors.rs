use std::io;

error_chain! {
  foreign_links {
    io::Error, IO;
  }

  errors {
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
