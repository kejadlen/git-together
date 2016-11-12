use std::io;

error_chain! {
  foreign_links {
    io::Error, IO;
  }

  errors {
    AuthorNotFound(initials: String) {
      description("author not found")
      display("author not found: '{}'", initials)
    }
    InvalidAuthor(initials: String, raw: String) {
      description("invalid author")
      display("invalid author for '{}': '{}'", initials, raw)
    }
  }
}
