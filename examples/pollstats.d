/*
 * This script profiles the following:
 * - distribution of poll durations across all tasks
 * - distribution of total duration for which a task exists
 * - distribution of total duration for which a task was actively being polled
 */


tokio$1:::task-spawn
{
    task_poll_times[arg0] = 0;
    task_spawn_times[arg0] = timestamp;
}

tokio$1:::task-poll-start
{
    self->poll_start_ts = timestamp;
}

tokio$1:::task-poll-end
/self->poll_start_ts/
{
    duration = timestamp - self->poll_start_ts;
    @durations["task poll duration"] = quantize(duration);
    task_poll_times[arg0] += duration;
    self->poll_start_ts = 0;
}

tokio$1:::task-terminate
{
    @durations["task total lifetime"] = quantize(timestamp - task_spawn_times[arg0]);
    @durations["task active time"] = quantize(task_poll_times[arg0]);
}
