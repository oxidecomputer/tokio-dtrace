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
