# Work Loop Tool Index

This is a navigation page. Each linked page documents exactly one tool with description and example usage.

- [`focusa_work_loop_writer_status`](tools/focusa_work_loop_writer_status.md)
- [`focusa_work_loop_status`](tools/focusa_work_loop_status.md)
- [`focusa_work_loop_control`](tools/focusa_work_loop_control.md)
- [`focusa_work_loop_context`](tools/focusa_work_loop_context.md)
- [`focusa_work_loop_checkpoint`](tools/focusa_work_loop_checkpoint.md)
- [`focusa_work_loop_select_next`](tools/focusa_work_loop_select_next.md)

Work-loop status tools use the bounded summary route by default; replay and deep diagnostics are cold paths and must be opt-in. The hot `/v1/work-loop/health` surface reports dispatch readiness, boundary reason, pause flags, and transport degradation so agents can decide whether to inspect writer/status/deep before dispatching or retrying.
