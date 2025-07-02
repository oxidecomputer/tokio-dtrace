#!/usr/sbin/dtrace -s
/*
 * When a task is spawned, count the stack that spawned it.
 */
tokio$1:::task-spawn
{
    @tasks[ustack()] = count();
}
