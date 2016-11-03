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
  }
}
