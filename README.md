# tokio-dtrace

[DTrace] probes for [Tokio].

## Overview

This crate provides a set of DTrace [USDT probes] for the Tokio async runtime,
using Tokio's runtime hooks. Probes are provided for the following events:

- **`tokio*:::task-spawn`: Records when a new [Tokio task] is [spawned].**

  `arg0` is the [task `Id`] of the spawned task.
- **`tokio*:::task-poll-start`: Records when the Tokio runtime begins [polling]
  the [`Future`] for a given task.**
  
  `arg0` is the [task `Id`] of the task being polled.
  
  Along with the `tokio*:::task-poll-end` probe, this probe may be used to
  determine the task ID of the currently running task on a given thread. This
  may, in turn, provide context to other DTrace probes that fire during that
  task's execution.
- **`tokio*:::task-poll-end`: Records when the Tokio runtime has finished
  [polling]  the [`Future`] for a given task.**
  
  `arg0` is the [task `Id`] of the task being polled.
  
  If polling the task returned [`Poll::Ready`] the poll, the 
  `tokio*:::task-terminate` probe will fire *before* the `task-poll-end`
  probe for that poll. Otherwise, if `task-terminate` does not fire, the task
  is still [pending].
- **`tokio*:::task-terminate`: Records when a task has terminated.**

  `arg0` is the  [task `Id`] of the task that has terminated.
- **`tokio*:::worker-thread-start`: Records when the runtime has started a new
  worker thread, but before it starts doing work.**
  
  This may be used to  determine if a given thread is a Tokio worker, or
  something else.
- **`tokio*:::worker-thread-stop`: Records when a worker thread is about to
  stop.**
- **`tokio*:::worker-thread-park`: Records when a worker thread is about to
  become idle because it has no tasks currently ready to poll.**
  
  Along with the `worker-thread-unpark` probe, this may be used to measure the
  utilization of worker threads.
- **`tokio*:::worker-thread-unpark`: Records when a parked worker thread
  unparks to begin performing work.**

A process that instruments its Tokio runtime using `tokio-dtrace` will
register a DTrace provider called `tokio${PID}` that is unique to that process.
In this example, PID 16687 is a process instrumented using this crate:

```console
eliza@atrium ~/tokio-dtrace $ pfexec dtrace -l -n tokio*:::
   ID   PROVIDER            MODULE                          FUNCTION NAME
14552 tokio16687             basic _ZN12tokio_dtrace5hooks18on_after_task_poll17h57940106588e89baE task-poll-end
14553 tokio16687             basic _ZN12tokio_dtrace5hooks19on_before_task_poll17h64ab06edcaab5d2bE task-poll-start
14554 tokio16687             basic _ZN12tokio_dtrace5hooks13on_task_spawn17hb99afbe81083cd1fE task-spawn
14555 tokio16687             basic _ZN12tokio_dtrace5hooks17on_task_terminate17hab4ccd3daf5dc91dE task-terminate
14556 tokio16687             basic _ZN12tokio_dtrace5hooks14on_thread_park17h216f5d29c208b65bE worker-thread-park
14557 tokio16687             basic _ZN12tokio_dtrace5hooks15on_thread_start17h43ad7d683fe84cf7E worker-thread-start
14566 tokio16687             basic _ZN12tokio_dtrace5hooks14on_thread_stop17h30e9720722e581fdE worker-thread-stop
14567 tokio16687             basic _ZN12tokio_dtrace5hooks16on_thread_unpark17hbba36de5355c6387E worker-thread-unpark
```

The following D script will print each `tokio-dtrace` probe as it fires:

```
tokio*:::task-poll-start,
tokio*:::task-poll-end,
tokio*:::task-spawn,
tokio*:::task-terminate
/pid == $1/
{
    printf("thread[%4d] %s(task=%d)\n", tid, probename, arg0);
}

tokio*:::worker-thread-start,
tokio*:::worker-thread-park,
tokio*:::worker-thread-unpark,
tokio*:::worker-thread-stop
/pid == $1/
{
    printf("thread[%4d] %s()\n", tid, probename);
}
```

More sophisticated tracing is also possible. For example, capturing a stack
trace in the `task-spawn` probe can be used to associate task IDs with the
stack frame in which the task was spawned, and the `task-poll-start` and
`task-poll-end` probes may be used to determine the task ID of the task
currently being polled when another probe is recorded. The duration of
individual task polls, total task runtime, and worker task idle time may also
be measured using these probes.

[DTrace]: https://illumos.org/books/dtrace/
[Tokio]: https://docs.rs/tokio
[USDT probes]: https://illumos.org/books/dtrace/chp-usdt.html#chp-usdt
[Tokio task]: https://docs.rs/tokio/latest/tokio/task/index.html
[spawned]: https://docs.rs/tokio/latest/tokio/task/fn.spawn.html
[task `Id`]: https://docs.rs/tokio/latest/tokio/task/struct.Id.html
[polling]: https://doc.rust-lang.org/stable/std/future/trait.Future.html#tymethod.poll
[`Future`]: https://doc.rust-lang.org/stable/std/future/trait.Future.html
[`Poll::Ready`]: https://doc.rust-lang.org/stable/std/task/enum.Poll.html#variant.Ready
[pending]:  https://doc.rust-lang.org/stable/std/task/enum.Poll.html#variant.Pending
