//! Fallible rendering for `bg list`; other command output is unaffected.
use serde_json::Value;
use std::io::{self, Write};

pub(super) fn write_list(
    mut output: impl Write,
    result: &Value,
    json_mode: bool,
) -> io::Result<()> {
    let written = (|| {
        if json_mode {
            let encoded = serde_json::to_string_pretty(result).map_err(io::Error::other)?;
            writeln!(output, "{encoded}")?;
        } else {
            for job in result
                .get("jobs")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let id = job.get("job_id").and_then(Value::as_str).unwrap_or("?");
                let name = job.get("name").and_then(Value::as_str).unwrap_or("?");
                let status = job.get("status").and_then(Value::as_str).unwrap_or("?");
                writeln!(output, "{id}\t{status}\t{name}")?;
            }
        }
        output.flush()
    })();
    match written {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct FailedOutput {
        kind: io::ErrorKind,
        on_flush: bool,
    }
    impl Write for FailedOutput {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.on_flush {
                Ok(bytes.len())
            } else {
                Err(self.kind.into())
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(self.kind.into())
        }
    }

    #[test]
    fn preserves_text_and_json_output() {
        let result = json!({"jobs": [{"job_id":"one","status":"completed","name":"proof"}, {}]});
        let mut output = Vec::new();
        write_list(&mut output, &result, false).unwrap();
        assert_eq!(output, b"one\tcompleted\tproof\n?\t?\t?\n");
        output.clear();
        write_list(&mut output, &result, true).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            format!("{}\n", serde_json::to_string_pretty(&result).unwrap())
        );
        let mut empty = Vec::new();
        write_list(&mut empty, &json!({"jobs": []}), false).unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn only_broken_pipe_is_clean_termination_in_both_modes() {
        for json_mode in [false, true] {
            for on_flush in [false, true] {
                for kind in [
                    io::ErrorKind::BrokenPipe,
                    io::ErrorKind::PermissionDenied,
                    io::ErrorKind::Other,
                ] {
                    let result = write_list(
                        FailedOutput { kind, on_flush },
                        &json!({"jobs":[{}]}),
                        json_mode,
                    );
                    if kind == io::ErrorKind::BrokenPipe {
                        assert!(result.is_ok());
                    } else {
                        assert_eq!(result.unwrap_err().kind(), kind);
                    }
                }
            }
        }
    }
}
